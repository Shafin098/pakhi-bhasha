## Introduction
Pakhi is a dynamically typed, general purpose programming language with Bangla alphabet written in rust.
## Language basics
#### PrimitiveTypes
- __সংখ্যা_
- __বুলিয়ান_
- __স্ট্রিং_
- __লিস্ট_
- __রেকর্ড_
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
#### List
```
নাম সংখ্যা = [১, ২, ৩, ৪, ৫];
দেখাও সংখ্যা[০];
```
#### Record
```
নাম তথ্য =  @{
    "নাম" -> "সিফাত",
    "বয়স" -> ৪২,
    "ফোন-নাম্বার" -> ["০১৭১১১১১১১১", "০১৭৩৩৩৩৩৩৩৩"],
};
দেখাও তথ্য["নাম"];
দেখাও তথ্য["বয়স"];
দেখাও তথ্য["ফোন-নাম্বার"];
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
#### Importing modules
Every pakhi source file is a module. Module can be imported with realtive path or absolute path.<br>
Module import syntax: ```মডিউল মডিউল-নাম = "মডিউল-পাথ";```<br>
Module use syntax: ```মডিউল-নাম/মডিউলের-ফাংশন();```
```
# এটা হচ্ছে রুট মডিউল ফাইল: main.pakhi #
মডিউল ম্যাথ = "math.pakhi";
নাম বাহু = ৩;
নাম ক্ষেত্রফল = ম্যাথ/বর্গ(বাহু);
দেখাও ক্ষেত্রফল;
```
```
# 
এটা হচ্ছে ম্যাথ মডিউল ফাইল: math.pakhi
রুট মডিউল এই ফাইল এর বর্গ ফাংশনকে ইম্পোর্ট করেছে। 
#
ফাং বর্গ(সংখ্যা) {
    ফেরত সংখ্যা * সংখ্যা;
} ফেরত;
```
#### Comment block
```
# এক লাইন কমেন্ট #

# 
মালটি লাইন
কমেন্ট
#

# রেকর্ড ডিক্লেয়ার করা #
নাম তথ্য =  @{
    "নাম" -> "সিফাত",
    "বয়স" -> ৪২,
    "ফোন-নাম্বার" -> ["০১৭১১১১১১১১", "০১৭৩৩৩৩৩৩৩৩"],
};
#
রেকর্ড থেকে ব্যক্তির
নাম, বয়স, ফোন তথ্য
প্রিন্ট করা
#
দেখাও তথ্য["নাম"];
দেখাও তথ্য["বয়স"];
দেখাও তথ্য["ফোন-নাম্বার"];
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
* Download only [pakhi-setup.exe](https://github.com/Shafin098/pakhi-bhasha/releases) from Assets
* Complete setup
* Write a pakhi program (Use any example from above)
* Open Pakhi and browse source file
* Run your program (extension should be .pakhi)
* To run pakhi program from cmd or powershell add ```C:\Program Files\Pakhi\bin``` to your path environment variable. ***Note: Bangla text will not properly show***
## Supported by JetBrains
<a href="https://www.jetbrains.com/?from=pakhi-bhasha" target="_blank"><img src="https://raw.githubusercontent.com/Shafin098/pakhi-bhasha/master/svg/jetbrains.svg?raw=true"></a>
