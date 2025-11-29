# Typst2Latex 

Typst2Latex is an automatic converter from unequivocal-ams typst documents to latex. 

It should be run in terminal with the following command :  
cargo run main.typ -b refs.bib
-b allows to add a bib file to tell apart the typst references that links to environments of the current documents to the one that link to a bibliographic reference since both of them are written with @ in typst.  

* A line should be skiped at the end of each environment.
* theorem-like environments should follow the following structure  
&nbsp;  
\#example[ExampleTitle  
`⏎`  
ExampleContent  
]

* Some typst environment might not be translated with this method. In this case one can escape some Latex code in the typst document with the following syntax  
&nbsp;  
// BEGIN NO TEX  
`⏎`  
My not-translatable typst code  
`⏎`  
// END NO TEX  
`⏎`  
/* BEGIN TEX  
My manual Latex translation  
\END TEX */  
&nbsp;  
* Grid inside figures should be places inside brackets with "\#grid"

