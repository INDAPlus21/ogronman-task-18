# ogronman-task-18


### **Run the program** 


~~You have to extract both korpus.txt and token.txt from their respective zip files for the program to work... (trying to understand how to access zipped files in rust)~~

No extraction is needed, just run the program first with `load` then with `find "word"`

```bash
cargo run -- load
cargo run -- find "word"
```

You can also run:
```bash
cargo run -- load debug
```
To get extra information

It is possible to do `cargo run -- find (word)` without first doing `cargo run -- load` however that will just run the code for generating the magic file

If reader.stream_len() worked this code would work with probably all files..

### **Benchmarks**

With the current code there are **3681** collisions, which is kinda okay since there are 400 000 different words (however this number should probably be a little bit higher since b and c does not collide however the difference is to small to fit all of b's byte-index before c's hashed posistion)

The time get most indices is about **50** -> **500** µs however for most single letter words like a, b, c and so on (words with many collisions) the time can go up to a couple of milliseconds.

The total time (get indices and information from korpus-file) depend on how many instances of that word exists. For words that occur often in korpus.txt it can take up to 20 ms, however it usually lies around 1-10 ms.