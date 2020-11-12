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
#### Builtin functions
```_রিড-লাইন()``` reads a line from stdin
```
দেখাও "কিছু টাইপ করুনঃ ";
নাম ক = _রিড-লাইন();
দেখাও ক;
```
```_লিস্ট-পুশ(লিস্ট, মান)``` adds a new element to end of a list
```
নাম লিস্ট = [১, ২, ৩];
_লিস্ট-পুশ(লিস্ট, ৪);
দেখাও লিস্ট;
```
```_লিস্ট-পুশ(লিস্ট, ইন্ডেক্স, মান)``` adds a new element at specific index 
```
নাম লিস্ট = [১, ২, ৩];
_লিস্ট-পুশ(লিস্ট, ০, ৪);
দেখাও লিস্ট;
```
```_লিস্ট-পপ(লিস্ট)``` removes last element from list
```
নাম লিস্ট = [১, ২, ৩];
_লিস্ট-পপ(লিস্ট);
দেখাও লিস্ট;
```
```_লিস্ট-পপ(লিস্ট, ইন্ডেক্স)``` removes element at a specific index
```
নাম লিস্ট = [১, ২, ৩];
_লিস্ট-পপ(লিস্ট, ১);
দেখাও লিস্ট;
```
## How to get pakhi on my computer?
* Download [pakhi-setup.exe](https://github.com/Shafin098/pakhi-bhasha/releases)
* Complete setup
* Write a pakhi program (Use any example from above)
* Open Pakhi Runner and browse source file
* Run your program (extension should be .pakhi)
* To run pakhi program from cmd or powershell add ```C:\Program Files\Pakhi\bin``` to your path environment variable. ***Note: Bangla text will not properly show***