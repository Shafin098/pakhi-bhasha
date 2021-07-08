## <a href="#">Introduction</a>
Pakhi is a dynamically typed, general purpose programming language with Bangla alphabet written in rust.
## <a href="#">Language basics</a>
***
#### <a href="#">PrimitiveTypes</a>
- __সংখ্যা_
- __বুলিয়ান_
- __স্ট্রিং_
- __লিস্ট_
- __রেকর্ড_
- __ফাং_
- __শূন্য_
#### <a href="#">Variable declaration</a>
```
নাম মাস = ১;
```
#### <a href="#">Print statement</a>
```
দেখাও মাস;
```
#### <a href="#">If-else statement</a>
```
যদি মাস == ১ {
    দেখাও "জানুয়ারি";
} অথবা {
    দেখাও "জানা নেই";
}
```
#### <a href="#">List</a>
```
নাম সংখ্যা = [১, ২, ৩, ৪, ৫];
দেখাও সংখ্যা[০];
```
#### <a href="#">Record</a>
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
#### <a href="#">Loop statement</a>
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
#### <a href="#">Function declaration and function call</a>
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
#### <a href="#">Importing modules</a>
Every pakhi source file is a module. Module can be imported with realtive path or absolute path.<br>
*Module import syntax: ```মডিউল মডিউল-নাম = "মডিউল-পাথ";```*<br>
*Module use syntax: ```মডিউল-নাম/মডিউলের-ফাংশন();```*
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
#### <a href="#">Comment block</a>
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
#### <a href="#">Builtin functions and constants</a>
***
* [_ডাইরেক্টরি](user_docs/built-in_functions_and_constants.md)
* [_প্ল্যাটফর্ম](user_docs/built-in_functions_and_constants.md)
* [_রিড-লাইন()](user_docs/built-in_functions_and_constants.md)
* [_স্ট্রিং(মান)](user_docs/built-in_functions_and_constants.md)
* [_সংখ্যা(মান)](user_docs/built-in_functions_and_constants.md)
* [_লিস্ট-পুশ(লিস্ট, মান)](user_docs/built-in_functions_and_constants.md)
* [_লিস্ট-পুশ(লিস্ট, ইন্ডেক্স, মান)](user_docs/built-in_functions_and_constants.md)
* [_লিস্ট-পপ(লিস্ট)](user_docs/built-in_functions_and_constants.md)
* [_লিস্ট-পপ(লিস্ট, ইন্ডেক্স)](user_docs/built-in_functions_and_constants.md)
* [_লিস্ট-লেন(লিস্ট)](user_docs/built-in_functions_and_constants.md)
* [_এরর("এরর মেসেজ")](user_docs/built-in_functions_and_constants.md)
* [_স্ট্রিং-স্প্লিট(string, split-by)](user_docs/built-in_functions_and_constants.md)
* [_স্ট্রিং-জয়েন(list-of-string, join-by)](user_docs/built-in_functions_and_constants.md)
* [_টাইপ(value)](user_docs/built-in_functions_and_constants.md)
* [_রিড-ফাইল("ফাইল-প্যাথ")](user_docs/built-in_functions_and_constants.md)
* [_রাইট-ফাইল("ফাইল-প্যাথ", "কন্টেন্ট")](user_docs/built-in_functions_and_constants.md)
* [_ডিলিট-ফাইল("ফাইল-প্যাথ", "কন্টেন্ট")](user_docs/built-in_functions_and_constants.md)
* [_নতুন-ডাইরেক্টরি("ডাইরেক্টরি-প্যাথ")](user_docs/built-in_functions_and_constants.md)
* [_রিড-ডাইরেক্টরি("ডাইরেক্টরি-প্যাথ")](user_docs/built-in_functions_and_constants.md)
* [_ডিলিট-ডাইরেক্টরি("ডাইরেক্টরি-প্যাথ")](user_docs/built-in_functions_and_constants.md)
* [_ফাইল-নাকি-ডাইরেক্টরি("প্যাথ")](user_docs/built-in_functions_and_constants.md)

## How to get pakhi on my computer?
* Download only [pakhi-setup.exe](https://github.com/Shafin098/pakhi-bhasha/releases) from Assets
* Complete setup
* Write a pakhi program (Use any example from above)
* Open Pakhi and browse source file
* Run your program (extension should be .pakhi)
* To run pakhi program from cmd or powershell add ```C:\Program Files\Pakhi\bin``` to your path environment variable. ***Note: Bangla text will not show properly***
## Supported by JetBrains
<a href="https://www.jetbrains.com/?from=pakhi-bhasha" target="_blank"><img src="https://raw.githubusercontent.com/Shafin098/pakhi-bhasha/master/svg/jetbrains.svg?raw=true"></a>
