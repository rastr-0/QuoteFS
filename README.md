# Quote File System
## Description
QuoteFS is a very basic file system that utilizes [fuser](https://crates.io/crates/fuser) crate for implementing File System in userspace.
My system generates a new quote of the famous people in the file ```random_quote.txt``` every time lookup function is triggered.
For example, this function triggers when ```stat``` command is run on the file. During this trigger are happening following things:
* Python script runs and uses ninjas API for retrieving quote of someone famous
* Rust code gets the execution result of the script and updates ```QuoteFS``` struct variables: ```file_content && file_size```
When user triggers read function with, for example, ```cat``` command he gets that one quote which python script retrieve via the API
That's pretty much all the unique features this file system has for this moment of time. Other available commands work in the usual way.
## Usage
For using QuoteFS you need
* Get an API_KEY on the [ninja web-site](https://api-ninjas.com) and save it to the file ```.env``` with the name ```KEY```
* Clone all the code on your local PC with ```git clone https://github.com/rastr-0/QuoteFS```
* Create a directrory which will be a mount point for your file system, let it be <b>mount_point</b>: ```mkdir mount_point```
* Compile the code to a binary with ```cargo build --verbose```
* Run the binary with ```./target/debug/QuoteFS mount_point/```

These steps will create your own File System in Userspace without changing a Linux kernel code! 
