#!/usr/bin/env python3
"""
Typst to LaTeX converter
Extracts title, authors, abstract, and bibliography from Typst files
and converts content sections, citations, and references

TODO: debug figures, images and math unsing pandoc
"""

import re
import sys
import argparse
from pathlib import Path
from typing import Dict, List, Optional, Tuple
import subprocess
import tempfile

def convert_math_with_pandoc(content: str) -> str:
    """
    Convert Typst math blocks to LaTeX using pandoc
    Handles both inline math $...$ and display math blocks
    """
    
    def convert_math_block(match):
        math_content = match.group(0)
        
        try:
            # Create a temporary file with the math content
            with tempfile.NamedTemporaryFile(mode='w', suffix='.typ', delete=False) as f:
                f.write(math_content)
                temp_input = f.name
            
            # Use pandoc to convert from typst to latex
            result = subprocess.run(
                ['pandoc', '-f', 'typst', '-t', 'latex', temp_input],
                capture_output=True,
                text=True,
                timeout=10  # 10 second timeout
            )
            
            # Clean up temp file
            Path(temp_input).unlink()
            
            if result.returncode == 0:
                return remove_trailing_newline(result.stdout)
            else:
                print(f"Pandoc error for math: {math_content[:100]}...")
                return math_content
        
        except Exception as e:
            print(f"Error converting math with pandoc: {e}")
            return math_content  # Fallback to original
    
    # Convert math
    content = re.sub(
        r'\$(.*?)\$',
        convert_math_block,
        content,
        flags=re.DOTALL
    )
    
    return content

def remove_trailing_newline(s: str) -> str:
    """
    Remove the last newline character from a string if it exists
    """
    if s.endswith('\n'):
        return s[:-1]
    return s

def convert_figures(content: str) -> str:
    """
    Convert Typst figure environments to LaTeX figure environments
    """
    
    def replace_figure(match):
        figure_content = match.group(1).strip()
        caption_content = match.group(2).strip() if match.group(2) else ""
        
        # Process the content (remove brackets, handle nested content)
        figure_content = re.sub(r'^\s*\[\s*|\s*\]\s*$', '', figure_content)
        
        # Build LaTeX figure environment
        latex_figure = []
        latex_figure.append(r'\begin{figure}[htbp]')
        latex_figure.append(r'\centering')
        
        # Add figure content (this could be images, tikz, etc.)
        if figure_content:
            # For now, just pass through the content - could add image conversion later
            latex_figure.append(figure_content)
        
        # Add caption if provided
        if caption_content:
            latex_figure.append(f'\\caption{{{caption_content}}}')
        
        latex_figure.append(r'\end{figure}')
        
        return '\n'.join(latex_figure)
    
    # Pattern for figure with caption
    content = re.sub(
        r'#figure\s*\(\s*\[\s*(.*?)\s*\]\s*caption:\s*(.*?)\s*\)',
        replace_figure,
        content,
        flags=re.DOTALL
    )
    
    # Pattern for figure without caption
    content = re.sub(
        r'#figure\s*\(\s*\[\s*(.*?)\s*\]\s*\)',
        lambda m: f'\\begin{{figure}}[htbp]\n\\centering\n{m.group(1).strip()}\n\\end{{figure}}',
        content,
        flags=re.DOTALL
    )
    
    return content

def remove_typst_commands(content: str) -> str:
    """
    Remove Typst commands (lines starting with #) and their arguments,
    including multi-line commands with parentheses and nested structures.
    """
    lines = content.split('\n')
    cleaned_lines = []
    
    i = 0
    while i < len(lines):
        line = lines[i]
        stripped = line.strip()
        
        # Check if this line starts with a command we want to remove
        if stripped.startswith('#'):
            # Check if it's a command we want to ignore
            if (stripped.startswith('#show') or 
                stripped.startswith('#set') or
                stripped.startswith('#import') or
                stripped.startswith('#outline') or
                stripped.startswith('#grid') or
                stripped.startswith('#pagebreak') or
                stripped.startswith('#let')) :
                
                # Count parentheses to find the end of the command
                paren_count = stripped.count('(') - stripped.count(')')
                skip_count = 0
                
                # If we have unmatched opening parentheses, continue scanning
                if paren_count > 0:
                    j = i + 1
                    while j < len(lines) and paren_count > 0:
                        next_line = lines[j]
                        paren_count += next_line.count('(') - next_line.count(')')
                        skip_count += 1
                        j += 1
                
                # Skip all lines that are part of this command
                i += skip_count + 1
                cleaned_lines.append('\n')
                continue
            else:
                # Keep other # commands (like headings, theorems, etc.)
                cleaned_lines.append(line)
        else:
            # Keep non-command lines
            cleaned_lines.append(line)
        
        i += 1
    
    return '\n'.join(cleaned_lines)

def convert_text_styling(content: str) -> str:
    """
    Convert Typst text styling commands to LaTeX
    """
    # Convert #text(style: "italic")[content] to \emph{content}
    content = re.sub(
        r'#text\s*\(\s*style:\s*"italic"\s*\)\s*\[([^\]]+)\]', 
        r'\\emph{\1}', 
        content
    )
    
    # Convert #text(style: "bold")[content] to \textbf{content}
    content = re.sub(
        r'#text\s*\(\s*style:\s*"bold"\s*\)\s*\[([^\]]+)\]', 
        r'\\textbf{\1}', 
        content
    )
    
    # Convert generic #text commands with other styles
    content = re.sub(
        r'#text\s*\([^)]+\)\s*\[([^\]]+)\]', 
        r'\1', 
        content
    )
    
    return content

def process_tex_comments(content: str) -> str:
    """
    Process special comments for LaTeX conversion:
    - Remove content between // BEGIN NO TEX and // END NO TEX
    - Extract and insert LaTeX code between // BEGIN TEX and // END TEX
    """
    # Remove NO TEX blocks (complete removal)
    content = re.sub(
        r'//\s*BEGIN NO TEX\s*.*?//\s*END NO TEX\s*',
        '',
        content,
        flags=re.DOTALL
    )

    # Process TEX blocks - capture the content between BEGIN TEX and END TEX
    def replace_tex_block(match):
        inner_content = match.group(1)  # Content between the comments
        # Check if the inner content is wrapped in /* */
        tex_content_match = re.search(r'/\*\s*(.*?)\s*\*/', inner_content, re.DOTALL)
        if tex_content_match:
            return tex_content_match.group(1).strip()
        else:
            return inner_content.strip()
    
    content = re.sub(
        r'//\s*BEGIN TEX\s*(.*?)\s*//\s*END TEX\s*',
        replace_tex_block,
        content,
        flags=re.DOTALL
    )
    
    return content

def convert_theorem_environments(content: str) -> str:
    """
    Convert Typst theorem-like environments to LaTeX with bracket balancing
    Uses first line of content as the title
    """
    
    def extract_title_from_content(env_content):
        """Extract the first line from content to use as title"""
        lines = env_content.split('\n')
        first_line = lines[0].strip() if lines else ''
        remaining_content = '\n'.join(lines[1:]).strip()
        return first_line, remaining_content
    
    env_patterns = [
        ('theorem', '#theorem'),
        ('proposition', '#proposition'), 
        ('lemma', '#lemma'),
        ('corollary', '#corollary'),
        ('proof', '#proof'),
        ('definition', '#definition'),
        ('example', '#example'),
        ('remark', '#remark')
    ]
    
    result = []
    i = 0
    n = len(content)
    
    while i < n:
        # Look for any theorem environment
        found = False
        for env_name, pattern in env_patterns:
            if content.startswith(pattern, i):
                # Found environment start
                env_start = i
                i += len(pattern)
                
                # Skip whitespace
                while i < n and content[i].isspace():
                    i += 1
                
                # Look for opening bracket
                if i < n and content[i] == '[':
                    bracket_start = i
                    bracket_count = 1
                    i += 1
                    content_start = i
                    
                    # Find matching closing bracket
                    while i < n and bracket_count > 0:
                        if content[i] == '[':
                            bracket_count += 1
                        elif content[i] == ']':
                            bracket_count -= 1
                        i += 1
                    
                    if bracket_count == 0:
                        # Successfully found balanced brackets
                        env_content = content[content_start:i-1]  # -1 to exclude closing ]
                        
                        # Extract title from first line (except for proof)
                        if env_name == 'proof':
                            # Proof doesn't get a title
                            latex_env = f'\\begin{{{env_name}}}\n{env_content}\n\\end{{{env_name}}}'
                        else:
                            title, remaining_content = extract_title_from_content(env_content)
                            if title and remaining_content:
                                latex_env = f'\\begin{{{env_name}}}{{{title}}}\n{remaining_content}\n\\end{{{env_name}}}'
                            elif title:
                                latex_env = f'\\begin{{{env_name}}}{{{title}}}\n\\end{{{env_name}}}'
                            else:
                                latex_env = f'\\begin{{{env_name}}}\n{env_content}\n\\end{{{env_name}}}'
                        
                        result.append(latex_env)
                        found = True
                        break
                
                # If we get here, parsing failed, revert
                i = env_start
                break
        
        if not found:
            result.append(content[i])
            i += 1
    
    return ''.join(result)

def fix_label_placement(content: str) -> str:
    
    # Pattern to match \end{env}\label{label} with optional whitespace variations
    pattern = r'(\\end\{(theorem|proposition|lemma|corollary|definition|example|remark|proof)\})\s*(\\label\{[^}]+\})'
    
    # Replace: move label before end
    content = re.sub(pattern, r'\3\1', content)
    
    return content


def load_citation_keys(bib_file: str) -> set:
    """
    Load all citation keys from a .bib file
    """
    citation_keys = set()
    
    if not bib_file or not Path(bib_file).exists():
        return citation_keys
    
    try:
        with open(bib_file, 'r', encoding='utf-8') as f:
            bib_content = f.read()
        
        # Pattern to match citation keys: @type{key,
        pattern = r'@\w+\{([^,]+),'
        matches = re.findall(pattern, bib_content)
        citation_keys.update(matches)
        
    except Exception as e:
        print(f"Warning: Could not read bibliography file {bib_file}: {e}")
    
    return citation_keys

def convert_citations_with_check(content: str, citation_keys: set) -> str:
    """
    Convert @citations to either \\cite or \\ref based on whether they exist in bibliography
    """

    
    def replace_citation(match):
        citation_key = match.group(1)
        if citation_key in citation_keys:
            return f'\\cite{{{citation_key}}}'
        else:
            return f'\\thref{{{citation_key}}}'
    
    # Convert citations with check
    content = re.sub(r'@([\w-]+)', replace_citation, content)
    return content

def convert_typst_to_latex_content(typst_content: str, citation_keys: set) -> str:
    """
    Convert Typst content to LaTeX content, handling sections, citations, etc.
    """
    # First remove commands we want to ignore
    
    content = remove_typst_commands(typst_content)

    # Convert math blocks using pandoc
    content = convert_math_with_pandoc(content)

    content = process_tex_comments(content)
    
    # Convert theorem environments
    content = convert_theorem_environments(content)

    content = convert_figures(content)
    
    # Convert text styling
    content = convert_text_styling(content)
    
    # Convert sections and subsections
    content = re.sub(r'^= (.*)$', r'\\section{\1}', content, flags=re.MULTILINE)
    content = re.sub(r'== (.*)$', r'\\subsection{\1}', content, flags=re.MULTILINE)
    content = re.sub(r'=== (.*)$', r'\\subsubsection{\1}', content, flags=re.MULTILINE)
    
    # Convert bold text
    content = re.sub(r'\[_\*([^*]+)\*_\]', r'\\textbf{\1}', content)
    content = re.sub(r'\*([^*]+)\*', r'\\textbf{\1}', content)
    
    # Convert italic text
    content = re.sub(r'\[_([^_]+)_\]', r'\\emph{\1}', content)
    content = re.sub(r'_([^_]+)_', r'\\emph{\1}', content)

    # Convert citations
    #content = re.sub(r'@([\w-]+)', r'\\citeref{\1}', content)
    content = convert_citations_with_check(content, citation_keys)
    
    # Convert references (simplified)
    content = re.sub(r'<([^>]+)>', r'\\label{\1}', content)
    
    content = fix_label_placement(content)
    
    # Remove remaining square brackets from Typst markup
    #content = re.sub(r'\[\[([^\]]+)\]\]', r'\1', content)
    #content = re.sub(r'\[([^\]]+)\]', r'\1', content)

    # Clean up multiple empty lines
    content = re.sub(r'\n\s*\n\s*\n', '\n\n', content)

    return content

def parse_typst_header(typst_content: str, citation_keys: set) -> Dict[str, str]:
    """
    Parse Typst header to extract title, authors, abstract, and bibliography
    """
    data = {
        'title': '',
        'authors': '',
        'abstract': '',
        'bibliography': '',
        'date': '\\today',
        'content': ''
    }
    
    # Extract title
    title_match = re.search(r'title:\s*\[([^\]]+)\]', typst_content)
    if title_match:
        data['title'] = title_match.group(1).strip()
    
    # Extract authors - handle multi-line author information
    authors_section = re.search(r'authors:\s*\(([^)]+)\)', typst_content, re.DOTALL)
    if authors_section:
        authors_text = authors_section.group(1)
        # Extract name
        name_match = re.search(r'name:\s*"([^"]+)"', authors_text)
        if name_match:
            data['authors'] = name_match.group(1).strip()
        
        # Extract affiliation
        affiliation_match = re.search(r'affiliation\s*:\s*"([^"]+)"', authors_text)
        if affiliation_match:
            if data['authors']:
                data['authors'] += '\\\\\n'
            data['authors'] += affiliation_match.group(1).strip()
    
    # Extract abstract
    abstract_match = re.search(r'abstract\s*:\s*(\w+)', typst_content)
    if abstract_match:
        abstract_var = abstract_match.group(1)
        # Look for the abstract variable definition
        abstract_def_match = re.search(rf'#let {abstract_var}\s*=\s*\[([^\]]+)\]', typst_content)
        if abstract_def_match:
            data['abstract'] = abstract_def_match.group(1).strip()
    
    # Extract bibliography
    bib_match = re.search(r'bibliography:\s*bibliography\("([^"]+)"\)', typst_content)
    if bib_match:
        data['bibliography'] = bib_match.group(1).strip()
    
    # Extract content (everything)
    data['content'] = convert_typst_to_latex_content(typst_content, citation_keys)
    
    return data

def create_latex_content(data: Dict[str, str]) -> str:
    """
    Create LaTeX content from extracted data
    """

    latex_template = r"""\documentclass[11pt,a4paper]{article}
\usepackage[utf8]{inputenc}
\usepackage[T1]{fontenc}
\usepackage{amsmath}
\usepackage{amsfonts}
\usepackage{amssymb}
\usepackage[thref]{ntheorem}
\usepackage{graphicx}
\usepackage{xcolor}
\usepackage[margin=1.5cm, top=3cm, bottom=2cm]{geometry}

\usepackage{biblatex}
\addbibresource{%(bibliography)s}


\newtheorem{theorem}{Theorem}
\newtheorem{proposition}{Proposition}
\newtheorem{lemma}{Lemma}
\newtheorem{corollary}{Corollary}
\newtheorem{definition}{Definition}
\newtheorem{example}{Example}
\newtheorem{remark}{Remark}

\title{%(title)s}
\author{%(authors)s}
\date{%(date)s}

\begin{document}

\maketitle

\begin{abstract}
%(abstract)s
\end{abstract}

%(content)s

\printbibliography

\end{document}
"""

    # Ensure all required keys exist with default values
    template_data = {
        'title': data.get('title', ''),
        'authors': data.get('authors', ''),
        'abstract': data.get('abstract', ''),
        'bibliography': data.get('bibliography', ''),
        'date': data.get('date', r'\today'),
        'content': data.get('content', '')
    }
    
    return latex_template % template_data

def main():
    parser = argparse.ArgumentParser(description='Convert Typst file to LaTeX file')
    parser.add_argument('input_file', help='Input Typst file (.typ)')
    parser.add_argument('-b', '--bibliography', help='Input Bibliography file (.bib)')
    parser.add_argument('-o', '--output', help='Output LaTeX file (.tex)')
    parser.add_argument('--verbose', '-v', action='store_true', help='Verbose output')
    
    args = parser.parse_args()
    
    # Check if input file exists
    input_path = Path(args.input_file)
    if not input_path.exists():
        print(f"Error: Input file '{args.input_file}' not found")
        sys.exit(1)

    if args.bibliography:
        bib_path = Path(args.bibliography)
        citation_keys = load_citation_keys(bib_path) if bib_path else set()
    else:
        citation_keys = set()
        
    # Determine output filename
    if args.output:
        output_path = Path(args.output)
    else:
        output_path = input_path.with_suffix('.tex')
    
    try:
        # Read Typst file
        with open(input_path, 'r', encoding='utf-8') as f:
            typst_content = f.read()
        
        if args.verbose:
            print("Parsing Typst header...")        
        
        # Parse header information
        data = parse_typst_header(typst_content, citation_keys)
        
        if args.verbose:
            print("Converting content to LaTeX...")
        
        # Create LaTeX content
        latex_content = create_latex_content(data)
    
        if args.verbose:
            print("Writting content to file...")
            
        # Write LaTeX file
        with open(output_path, 'w', encoding='utf-8') as f:
            f.write(latex_content)
        
        print(f"Successfully converted '{input_path}' to '{output_path}'")
        
        if args.verbose:
            print("Conversion summary:")
            print(f"  Title: {data['title'] or 'Not found'}")
            print(f"  Authors: {data['authors'] or 'Not found'}")
            print(f"  Abstract: {'Found' if data['abstract'] else 'Not found'}")
            print(f"  Bibliography: {data['bibliography'] or 'Not found'}")
        
    except Exception as e:
        print(f"Error during conversion: {e}")
        sys.exit(1)

if __name__ == '__main__':
    main()
