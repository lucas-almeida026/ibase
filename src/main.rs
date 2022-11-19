use std::env;
use std::fs::{OpenOptions, File};
use std::path::Path;
use std::io::{Write, Read};

fn split_path (path: &Path) -> Option<Vec<&str>> {
  path.to_str().map(|str| str.split("/").collect()) 
}

fn get_or_create_file (file_path: &Path) -> Result<File, std::io::Error> {
  let found = file_path.is_file();
  if found {
    OpenOptions::new()
    .write(true)
    .read(true)
    .append(true)
    .open(file_path)
  } else {
    File::create(file_path)
  }
}

fn calc_back_paths (parent_folders: &Vec<&str>, base_path: String, target_name: &str) -> Vec<[String; 2]> {
  let mut backtrack_paths: Vec<[String; 2]> = vec![];
  for i in 0..parent_folders.len() {
    let last = i == parent_folders.len() - 1;
    let sub_folders = &parent_folders[0..i + 1];

    let prev_subfolder = if i < parent_folders.len() - 1 {
      format!("{}", parent_folders[i + 1])
    } else {
      format!("{}", parent_folders[0])
    };
    let mut subfolders_joined = String::from(base_path.as_str());
    subfolders_joined.push_str(format!("{}{}", "/", sub_folders.join("/").as_str()).as_str());
    backtrack_paths.push([
      subfolders_joined,
      String::from(if last { target_name } else { prev_subfolder.as_str() })
    ]);
  };
  return backtrack_paths
}

fn appen_export_line (index_file_path: &Path, line: &String, line_core: Option<&str>) {
  let file = get_or_create_file(index_file_path);

  match file {
    Ok(mut f) => {
      let includes_line = if line_core.is_some() {
        let mut buffer = String::new();
        let read = f.read_to_string(&mut buffer);
        match read {          
          Ok(_) => buffer.contains(line_core.unwrap()),
          Err(e) => {
            println!("Error reading file: {}; Error: {}", index_file_path.to_str().unwrap(), e);
            false
          }
        }
      } else {
        false
      };

      if includes_line {
        println!("Skipped: {}", index_file_path.to_str().unwrap());
        return;
      }
      let res = writeln!(f, "{}", line);
      match res {
        Ok(_) => println!("Wrote to: {}", index_file_path.to_str().unwrap()),
        Err(e) => eprintln!("Error writing to file {}; Error: {:?};", index_file_path.to_str().unwrap(), e)
      }
    },
    Err(e) => eprintln!("Error opening file {};\nError: {:?};", index_file_path.to_str().unwrap(), e)
  }
}

fn main() {
  //FIXME: last folder is not considered as a base folder
  let args = env::args();
  if args.len() > 2 {
    let argsi: Vec<String> = args.collect();
    let target = &argsi[1];
    let base = &argsi[2];
    let target_path = Path::new(target);
    let base_path = Path::new(base);
    if !base_path.is_dir() {
      println!("Invalid argument <base>; Expecting a folder path");
      return;
    }
    if !target_path.is_file() {
      println!("Invalid argument <target>; Expecting a file path");
      return;
    }

    let target_path_splited = split_path(target_path);
    let base_path_splited = split_path(base_path);

    if target_path_splited.is_none() {
      format!("Error: conversion error (&Path -> Vec<&str>) @ <target>");
      return
    }

    if base_path_splited.is_none() {
      format!("Error: conversion error (&Path -> Vec<&str>) @ <base>");
      return
    }

    let base_path_meaningful_length = base_path_splited.as_ref().unwrap().len() - 1;
    let path_diff: &[&str] = &target_path_splited.unwrap()[base_path_meaningful_length..];
    let parent_folders = path_diff[0..path_diff.len() - 1].to_vec();

    let filename_with_ext = *path_diff.last().unwrap();
    let filename = &filename_with_ext[0..filename_with_ext.rfind(".").unwrap()];
    
    let base_path_usable = base_path_splited.unwrap();
    let cropped_base_path = base_path_usable[..base_path_usable.len() - 1].to_vec().join("/");
    let backtrack_paths = calc_back_paths(
      &parent_folders,
      cropped_base_path,
      filename
    );
    for i in 0..backtrack_paths.len() {
      let [target_path, export_path] = &backtrack_paths[i];
      let last = i == backtrack_paths.len() - 1;
      let formated_export_path = format!("\"./{}\"", export_path);
      let export_line = if last {
        format!("export {{default as {}}} from {}", filename, formated_export_path)
      } else {
        format!("export * from {}", formated_export_path)
      };
      let maybe_index = &format!("{}{}", target_path, "/index.ts");
      let line_core_string = format!("from {}", formated_export_path);
      let line_core = Some(line_core_string.as_str());
      appen_export_line(Path::new(maybe_index), &export_line, line_core);
    }
  } else {
    println!("Usage: ibase <target> <base>")
  }
}