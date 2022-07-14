## typedoc commenter

### Description

This is a standalone program written in rust in aid of typedoc.
Typedoc is a automatic comment program for documentation for typescript. 
However, it doesn't generate any documentation for the new typescript feature
decorator ` @decorator `, which is heavily used in our web service framework.
This program can be run on the codebase and generate corresponding
documentation for the ` @decorator `, so typedoc can process it. 

### Installation 

Installation: The package is on the internal npm registry: https://artifacts.co.ihc.com/repomgr/repository/npm-all
and the package name is *typedoc-commenter*.

Following command for installation:

```
npm config set registry https://artifacts.co.ihc.com/repomgr/repository/npm-all
npm install typedoc-commenter
```

### Usage

Using npx, you can call commenter program on a directory with file extension
```
npx commenter <dirname>/  -ends <file-extension>  
``` 
Following will run on `src/` with any `.dto.ts` file
e.g
```
npx commenter src/ -ends .dto.ts
```
You can also run it on individual file name
```
npx commenter <filename> 
```

### Failure

With the ts and js fast pace changing syntax, the program might face failure,
and could potentially illy overwritten the src code. But no worries, this
program runs a diff before overwritten your src code to make sure there is no
src code deletion and only comments are added.  

### Development

This code is written in rust, which is a robust system-level language. Feel
free to tinker around code. it is a relatively small project. 

### Author

Jerry Z.
