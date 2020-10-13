## Introduction
Pakhi is a dynamically typed, general purpose programming language with bangla script written in rust.
## Language basics
#### PrimitiveTypes
- __সংখ্যা_
- __বুলিয়ান_
- __স্ট্রিং_
- __লিস্ট_
- __ফাং_
- __শূন্য_
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
নাম সংখ্যা = [১, ২, ৩, ৪, ৫];
নাম ইন্ডেক্স = ০;
নাম যোগফল = ০;
লুপ {
    যদি ইন্ডেক্স > ৪ {
        থামাও;
    }
    যোগফল = যোগফল + সংখ্যা[ইন্ডেক্স];
    ইন্ডেক্স = ইন্ডেক্স + ১;
} আবার;
_দেখাও "ফলাফল = ";
দেখাও যোগফল;
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
* Get rar file from [pakhi-v0.2-alpha](https://github.com/Shafin098/pakhi-bhasha/releases)
* Extract and add full path to `pakhi-v0.2-alpha/bin` to your `Path` environment variable
* Open cmd and type `pakhi`. If no error shows pakhi was added to your path variable
* Write a pakhi program (Use any example from above)
* Run your program (extension should be .pakhi) with `pakhi source_file_name.pakhi` command (Note: use unicode supported command processor (not cmd or powershell), windows terminal or terminal in vs-code can be used)