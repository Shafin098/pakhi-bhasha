## Introduction
Pakhi is a dynamically typed, general purpose programming language with bangla script written in rust.
## Language basics
#### Variable declaration
```
নাম মাস = ১;
```
#### Print statement
```
দেখাও মাস;
```
#### If-else statement
```
যদি মাস == ১ {
    দেখাও "জানুয়ারি";
} অথবা {
    দেখাও "জানা নেই";
}
```
#### Loop statement
```
নাম কাউন্টার = ১;
লুপ {
  যদি কাউন্টার > ১০০ {
    থামাও;
  }
  দেখাও কাউন্টার;
  কাউন্টার = কাউন্টার + ১;
} আবার;
```
#### Function declaration and function call
```
ফাং জোড়(সংখ্যা) {
  যদি সংখ্যা % ২ == ০ {
    দেখাও "সংখ্যাটি জোড়";
  } অথবা {
    দেখাও "সংখ্যাটি বিজোড়";
  }
} ফেরত;

নাম স = ৪২;
জোড়(স);
```
## How to get pakhi on my computer?
* Get pre compiled binary from [pakhi.exe](https://github.com/Shafin098/pakhi-bhasha/releases)  Note: Download only pakhi.exe from Assets menu
* Add pakhi as your environment variable
* Write a pakhi program (Use example from above).
* Run your program (extension should be .pakhi) with `pakhi source_file_name.pakhi` command