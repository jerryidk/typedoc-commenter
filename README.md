## typedoc commenter

### Description

This is a standalone program written in rust in aid of typedoc.
Typedoc is a automatic comment program for documentation for typescript. 
However, it doesn't generate any documentation for the new typescript feature
decorator ` @decorator `, which is heavily used in our web service framework.
This program can be run on the codebase and generate corresponding
documentation for the ` @decorator `, so typedoc can process it. 

