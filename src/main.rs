
use std::io::prelude::*;
use std::fs;
use std::fs::File;
use std::path::Path;


use std::io::{self, BufRead, Write, Seek};
use std::io::BufReader;

use std::time::{Instant};

use clap::Parser;

use std::process;


#[derive(Parser, Clone)]
pub struct Cli {
    pattern: Vec<String>,
}
#[derive(Debug)]
pub struct Element{
    index: usize,
    key: usize,
}

impl Element{
    pub fn create(data: String, len: &usize, byte_index: usize) -> Element{
        let split_data = data.split_whitespace().map(|s| s.to_string()).collect::<Vec<String>>();
        let hashed_word: usize = hashlib::hash_string(&split_data[0]);
        //let data = split_data[1..].iter().map(|s| s.to_string() + " ").collect::<String>();
        Element{
            key: (hashed_word % len),
            index: byte_index,
        }
    }
}

fn main() {

    let args = Cli::parse();


    if args.pattern.len() < 1 {
        println!("");
        println!("error: no valid argument was given");
        println!("");
        process::exit(1);
    }

    match args.pattern[0].as_str() {
        "find" => {
            if args.pattern.len() > 1{
                find(&args.pattern[1]);
            }else{
                
            }
        },
        "load" => {

            let fname = Path::new("korpus.zip");
            let file = fs::File::open(&fname).unwrap();
            let reader = BufReader::new(file);
            let mut archive = zip::ZipArchive::new(reader).unwrap();
            zip::ZipArchive::extract(&mut archive, Path::new("")).unwrap();

            let start = Instant::now();
            if args.pattern.len() > 1 && args.pattern[1].eq("debug"){
                println!("Debugging...");
                create_magic_file(true);
            }else{
                create_magic_file(false);
            }
            let duration = start.elapsed();
            println!("It took: {:?} to generate the magic file", duration);
        },
        _ => (),
    }
}


//Read as Byterecord to speed things up...
fn find(in_word: &String){

    let word = in_word.to_lowercase();

    //Get/make bufreader
    let in_magic = Path::new("magic_file.txt");
    let in_index = Path::new("index_file.txt");
    let in_korpus = Path::new("korpus.txt");
    let mut file_reader_magic;
    let mut file_reader_index;
    let mut file_reader_korpus;
    match File::open(in_magic) {
        Ok(f) => {
            file_reader_magic = io::BufReader::new(f);  
            //file_reader = csv::Reader::from_reader(f);  
            //rdr = csv::Reader::from_reader(f);    
        },
        _ => {
            create_magic_file(false);
            find(&word);
            return;
        }
    };
    match File::open(in_index) {
        Ok(f) => {
            file_reader_index = io::BufReader::new(f);
            //file_reader = csv::Reader::from_reader(f);  
            //rdr = csv::Reader::from_reader(f);    
        },
        _ => {
            create_magic_file(false);
            find(&word);
            return;
        }
    };
    match File::open(in_korpus) {
        Ok(f) => {
            file_reader_korpus = io::BufReader::new(f);
            //file_reader = csv::Reader::from_reader(f);  
            //rdr = csv::Reader::from_reader(f);    
        },
        _ => {
            create_magic_file(false);
            find(&word);
            return;
        }
    };

    let index = hashlib::hash_string(&word) % 116502101;

    //assert_eq!(file_reader_index.stream_len().unwrap(), 116502101 as u64);

    let mut in_bin = vec![];

    let start = Instant::now();

        file_reader_magic.seek_relative((index - 1)  as i64).unwrap();
        file_reader_magic.read_until(b'%', &mut in_bin).unwrap();
        
       
        //If we are in the middle of a string
        if in_bin[0] == 37{
            in_bin = vec![];
            file_reader_magic.read_until(b'%', &mut in_bin).unwrap();
        }

        let mut buf = String::from_utf8_lossy(&in_bin[0..in_bin.len()-1]).parse::<usize>().unwrap();
        

        file_reader_index.seek_relative(buf as i64).unwrap();

        let mut in_word:String = "".to_string();
        file_reader_index.read_line(&mut in_word).unwrap();

        while !word.eq(&in_word[..word.len()]){
            let after = file_reader_index.stream_position().unwrap();
            file_reader_index.seek_relative(0 as i64 - after as i64).unwrap();
            in_bin = vec![];
            file_reader_magic.read_until(b'%', &mut in_bin).unwrap();
            buf = String::from_utf8_lossy(&in_bin[0..in_bin.len()-1]).parse::<usize>().unwrap();
            file_reader_index.seek_relative(buf as i64).unwrap();
            in_word = "".to_string();
            file_reader_index.read_line(&mut in_word).unwrap();
        }


        //Why is "a 22670403f" in the token.txt file...
        //That was very rude    
        let indexes: Vec<String> = in_word[word.len()..].split_whitespace().map(|s| s.to_string()).collect::<Vec<String>>();
        let duration = start.elapsed();
        println!("Duration to get all indexes: {:?}", duration);
        //let mut difference = 0;
        let offset = 12i64;
        let mut collection: Vec<String> = vec![];

        for ind in &indexes{
            let i = ind.parse::<usize>().unwrap_or(0);
            file_reader_korpus.seek_relative(i as i64 - offset);
            let mut in_buffer = vec![0; (offset*2i64) as usize];
            file_reader_korpus.read(&mut in_buffer).unwrap();
            //file_reader_korpus.read(b'.',&mut in_buffer);
            let in_value = String::from_utf8_lossy(&in_buffer).to_string();
            let after = file_reader_korpus.stream_position().unwrap();
            file_reader_korpus.seek_relative(0 as i64 - after as i64).unwrap();
            collection.push(in_value);
        }

    let duration = start.elapsed();

    println!("The word is: {:#?}", &in_word[..word.len()]);

    println!("{} instances was found", collection.len());
    //This for loop prints the context of every instance of the word
    //I disabled the printing because it makes it very hard to compare the time 
    for mut out in collection{
        out = out.replace("\n", " ");   
        println!("The context is: {}", out)
    }

    println!("The duration for everything was: {:?}", duration);

}





fn create_magic_file(debug: bool){

    create_index_file(debug);

    let in_path = Path::new("index_file.txt");
    let out_path = Path::new("magic_file.txt");
    
    let in_data = fs::read_to_string(in_path).expect("Unable to read file");
    let lines:Vec<String> = in_data.split("\n").map(|s| s.to_string().replace("\r", "")).collect::<Vec<String>>();


    if debug {
        for i in &lines{
            println!("File contains: {}", i);
        }
    }

    let mut bytes: usize = 0;
    //will become 116502101

    //There is probably a better way to do this..
    for i in &lines{
        bytes += i.len();
    }

    let mut new_data: Vec<Element> = vec![];
    let mut current_bytes: usize = 0;
    for u in &lines{
        new_data.push(Element::create(u.to_string(), &bytes, current_bytes));
        current_bytes += u.len() + 1;
    }

    
    new_data.sort_by_key(|a| a.key);

    let file = File::create(out_path).unwrap();
    let mut file_writer = io::LineWriter::new(file);
    let mut bytes_file: usize = 1;


    //For loop that will print the byte-index of every word to the magic-file where the byte-index of the word is (almost) the hashed word
    for i in 0..new_data.len(){
        //If and while the hashed-word index is bigger than the amount of bytes in the file, add more bytes to the file
            while new_data[i].key > bytes_file{
                file_writer.write("\n".as_bytes()).unwrap();
                bytes_file += 1;
            }
        
        let mut new_out:String = new_data[i].index.to_string();
        //Seperate evey byte-index with a % to make it easier to read the file, and makes collisions/cases of bytes and byte-index not lining up easier and quicker
        new_out += "%";
        file_writer.write(new_out.as_bytes()).unwrap();
        bytes_file += new_out.len();
    }

    file_writer.flush().unwrap();

    println!("Amount of lines {}", bytes);
    
    //I think this thing can determine the amount of collisions but I dont really know...
    if debug {
        println!("{}", bytes);
        let mut collisions: Vec<usize> = vec![];
        println!("Getting collisions");
        for _i in 0..bytes{
            collisions.push(0);
        }

        for u in new_data{
            collisions[u.key] = collisions[u.key]+1;
        }


        let mut collision_counter = 0;
        for j in 0..collisions.len(){
            if !&collisions[j].eq(&1) && !&collisions[j].eq(&0){
                println!("Number is: {} at index {}", collisions[j], j);
                collision_counter += collisions[j];
            }
        }
        println!("There are currently {} collisions", collision_counter);
        println!("{}", bytes);
        //println!("File contains: {:#?}", collisions);
    }


}


fn create_index_file(debug: bool){

    let out_val = Path::new("index_file.txt");

    if debug{
        println!("Reading input...");
    }

    // Get the token.txt from the token.zip file
    // Lets a bufferedreader read the txt.file that is compressed in the zip-file without having to extract it
    // This however comes with the drawback that some things / functions can not be used (or did not work for me :/ )
    let fname = std::path::Path::new("token.zip");
    let file = fs::File::open(&fname).unwrap();
    let reader = BufReader::new(file);
    let mut archive = zip::ZipArchive::new(reader).unwrap();
    let file = archive.by_index(0).unwrap();
    let mut in_lines = io::BufReader::new(file);
    // Get all of the lines in the token.txt file and collect them in a correctly strucured way (without \r and \n)
    let lines:Vec<String> = in_lines.lines().map(|s| s.unwrap().to_string().replace("\r", "")).collect::<Vec<String>>();

    
    if debug{
        for i in &lines{
            println!("File contains: {}", i);
        }
    }

    let mut new_data: String = "".to_string();
    let mut current_word: &str = &lines[0].split_whitespace().collect::<Vec<&str>>()[0];
    let mut posistions: String = "".to_string();

    for line in &lines{
        if line.len() < 1{
            break;
        }
        let words: Vec<&str> = line.split_whitespace().collect::<Vec<&str>>();

        if !current_word.eq(words[0]){
            //Adds the current word with all instances of that word to new_data followed by a newline
            new_data += current_word;
            new_data += &posistions;
            new_data += "\n";

            //Resets current_word and posistions
            current_word = words[0];
            posistions = "".to_string();

        }
        posistions += &(" ".to_owned() + words[1]);
    }
    //Add remaining data...
    new_data += current_word;
    new_data += &posistions;




    if debug{
        println!("New data is: \n{}", new_data);
    }

    write_file(out_val, new_data);

}


// loads a specific file from a path
// I dont think I use this function
pub fn load_file(path: &Path) -> Vec<String>{

    let mut lines: Vec<String> = vec![];

    match File::open(path){
        Ok(file) => {
            lines = io::BufReader::new(file).lines().map(|l| l.ok().unwrap()).collect();
        },
        _ => {
            println!("Could not find file...");
        },
    }

    lines
}

//Writes data to file
//I dont know why i have a function for this since it is only one line..
pub fn write_file(path: &Path, new_data: String){
    fs::write(path, new_data).expect("Unable to write file");
}
