#### <a href="#">Builtin constants</a>
***
### _ডাইরেক্টরি
###### *it will expand to source files directory path, it's primarily used when reading writing file or directory with relative path*
```
# This will error #
_রিড-ফাইল("../dir/file.txt");
# To fix previous error relative path must be concatenated with _ডাইরেক্টরি constant #
_রিড-ফাইল(_ডাইরেক্টরি + "../dir/file.txt");
```
### _প্ল্যাটফর্ম
###### *it will expand to user's os name*
###### *Possible values are*
- linux
- macos
- ios
- freebsd
- dragonfly
- netbsd
- openbsd
- solaris
- android
- windows

#### <a href="">Builtin functions</a>
***
### _রিড-লাইন()
###### *reads a line from stdin*
```
দেখাও "কিছু টাইপ করুনঃ ";
নাম ক = _রিড-লাইন();
দেখাও ক;
```

### _স্ট্রিং(মান)
###### *converts number to string*
```
নাম সংখ্যা = ৪২.৫৫;
দেখাও _স্ট্রিং(সংখ্যা); # "৪২.৫৫" #
```

### _সংখ্যা(মান)
###### *converts string to number*
```
নাম স্ট্রিং= "৪২.৫৫";
দেখাও _সংখ্যা(স্ট্রিং); # ৪২.৫৫ #
```

### _লিস্ট-পুশ(লিস্ট, মান)
###### *adds a new element to end of a list*
```
নাম লিস্ট = [১, ২, ৩];
_লিস্ট-পুশ(লিস্ট, ৪);
দেখাও লিস্ট; # [১, ২, ৩, ৪] #
```

### _লিস্ট-পুশ(লিস্ট, ইন্ডেক্স, মান)
###### *adds a new element at specific index*
```
নাম লিস্ট = [১, ২, ৩];
_লিস্ট-পুশ(লিস্ট, ০, ৪);
দেখাও লিস্ট; # [৪, ১, ২, ৩] #
```

### _লিস্ট-পপ(লিস্ট)
###### *removes last element from list*
```
নাম লিস্ট = [১, ২, ৩];
_লিস্ট-পপ(লিস্ট);
দেখাও লিস্ট; # [১, ২] #
```

### _লিস্ট-পপ(লিস্ট, ইন্ডেক্স)
###### *removes element at a specific index*
```
নাম লিস্ট = [১, ২, ৩];
_লিস্ট-পপ(লিস্ট, ১);
দেখাও লিস্ট; # [১, ৩] #
```

### _লিস্ট-লেন(লিস্ট)
###### *returns number of total elements in a list*
```
নাম লিস্ট = [১, ২, ৩];
দেখাও _লিস্ট-লেন(লিস্ট); # ৩ #
```

### _এরর("এরর মেসেজ")
###### *stops pakhi program and shows error message*

### _স্ট্রিং-স্প্লিট(string, split-by)
###### *splits a string by split-string and return splitted string as _লিস্ট*
```
নাম স্ট্রিং = "This-will-split-by-hyphen";
নাম স্প্লিটেড-স্ট্রিং = _স্ট্রিং-স্প্লিট(স্ট্রিং, "-")
দেখাও স্প্লিটেড-স্ট্রিং; # ["this", "will", "split", "by", "hyphen"] #
```

### _স্ট্রিং-জয়েন(list-of-string, join-by)
###### *joins a list of string and returns joined string*
```
নাম স্ট্রিং-লিস্ট = ["This", "will", "join", "by", "hyphen"];
নাম জয়েনড-স্ট্রিং = _স্ট্রিং-জয়েন(স্ট্রিং-লিস্ট, "-")
দেখাও জয়েনড-স্ট্রিং; # "This-will-join-by-hyphen" #
```

### _টাইপ(value)
###### *return type of any value in string format*
```
দেখাও _টাইপ(১); # "_সংখ্যা" #
দেখাও _টাইপ(মিথ্যা); # "_বুলিয়ান" #
দেখাও _টাইপ("string"); # "_স্ট্রিং" #
দেখাও _টাইপ([১, ১]); # "_লিস্ট" #
দেখাও _টাইপ(@{"১" -> ১,}); # "_রেকর্ড" #
নাম ক;
দেখাও _টাইপ(ক); # "_শূন্য" #
ফাং খ() {
} ফেরত;
দেখাও _টাইপ(খ); # "_ফাং" #
```

### _রিড-ফাইল("ফাইল-প্যাথ")
###### *reads a file from specified path and returns it's content*
```
দেখাও _রিড-ফাইল("E:/dir/file.txt");
# if path is relative, must use _ডাইরেক্টরি constant #
দেখাও _রিড-ফাইল(_ডাইরেক্টরি, "../dir/file.txt"); # reading a file which is one level above current directory #
```

### _রাইট-ফাইল("ফাইল-প্যাথ", "কন্টেন্ট")
###### *writes content to specified path, if file exists old contents is replaced and if file doesn't exist new file is created*
```
_রাইট-ফাইল("E:/dir/file.txt", "Hello, World!");
# if path is relative, must use _ডাইরেক্টরি constant #
_রাইট-ফাইল(_ডাইরেক্টরি, "../dir/file.txt", "Hello, World!"); # writing to a file which is one level above current directory #
```

### _ডিলিট-ফাইল("ফাইল-প্যাথ", "কন্টেন্ট")
###### *deletes a file specified by path*
```
_ডিলিট-ফাইল("E:/dir/file.txt");
# if path is relative, must use _ডাইরেক্টরি constant #
_ডিলিট-ফাইল(_ডাইরেক্টরি, "../dir/file.txt"); # delteting a file which is one level above current directory #
```

### _নতুন-ডাইরেক্টরি("ডাইরেক্টরি-প্যাথ")
###### *creates new directory specified by path*
```
_নতুন-ডাইরেক্টরি("E:/dir");
# if path is relative, must use _ডাইরেক্টরি constant #
_নতুন-ডাইরেক্টরি(_ডাইরেক্টরি, "../dir"); # creating a directory which is one level above current directory #
```

### _রিড-ডাইরেক্টরি("ডাইরেক্টরি-প্যাথ")
###### *returns all files and directory names*
```
_দেখাও রিড-ডাইরেক্টরি("E:/dir");
# if path is relative, must use _ডাইরেক্টরি constant #
_দেখাও রিড-ডাইরেক্টরি(_ডাইরেক্টরি, "../dir"); # reading from a directory which is one level above current directory #
```

### _ডিলিট-ডাইরেক্টরি("ডাইরেক্টরি-প্যাথ")
###### *deletes directory specified by path*
```
_ডিলিট-ডাইরেক্টরি("E:/dir");
# if path is relative, must use _ডাইরেক্টরি constant #
_ডিলিট-ডাইরেক্টরি(_ডাইরেক্টরি, "../dir"); # delete a directory which is one level above current directory #
```

### _ফাইল-নাকি-ডাইরেক্টরি("প্যাথ")
###### *returns "ফাইল" string if path is to a file or returns "ডাইরেক্টরি" string if path is to a directory*