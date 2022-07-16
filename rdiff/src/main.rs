use std::{
    cmp::max,
    env,
    error::Error,
    fs::File,
    io::{self, BufRead},
    process,
};

use grid::Grid;

pub mod grid;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("too few arguments.");
        process::exit(1);
    }
    let filename1 = &args[1];
    let filename2 = &args[2];

    let lines1 = read_file_lines(filename1)?;
    let lines2 = read_file_lines(filename2)?;

    let lcs_table = lcs(&lines1, &lines2);

    print_diff(&lcs_table, &lines1, &lines2, lines1.len(), lines2.len());
    Ok(())
}

fn read_file_lines(filename: &str) -> Result<Vec<String>, io::Error> {
    let mut res = vec![];
    let file = File::open(filename)?;
    for line in io::BufReader::new(file).lines() {
        res.push(line?);
    }
    Ok(res)
}

fn lcs(seq1: &Vec<String>, seq2: &Vec<String>) -> Grid {
    let mut grid = Grid::new(seq1.len() + 1, seq2.len() + 1);
    for i in 0..seq1.len() + 1 {
        grid.set(i, 0, 0).unwrap();
    }
    for j in 0..seq2.len() + 1 {
        grid.set(0, j, 0).unwrap();
    }
    for i in 0..seq1.len() {
        for j in 0..seq2.len() {
            if seq1[i] == seq2[j] {
                grid.set(i + 1, j + 1, grid.get(i, j).unwrap() + 1).unwrap();
            } else {
                grid.set(
                    i + 1,
                    j + 1,
                    max(grid.get(i, j + 1).unwrap(), grid.get(i + 1, j).unwrap()),
                )
                .unwrap();
            }
        }
    }
    grid
}

fn print_diff(lcs_table: &Grid, lines1: &Vec<String>, lines2: &Vec<String>, i: usize, j: usize) {
    if i > 0 && j > 0 && lines1[i - 1] == lines2[j - 1] {
        print_diff(lcs_table, lines1, lines2, i - 1, j - 1);
        println!("  {}",lines1[i - 1]);
    } else if j > 0 && (i == 0 || lcs_table.get(i, j - 1).unwrap() >= lcs_table.get(i - 1, j).unwrap()) {
        print_diff(lcs_table, lines1, lines2, i, j - 1);
        println!("> {}",lines2[j - 1]);
    } else if i > 0 && (j == 0 || lcs_table.get(i, j - 1).unwrap() < lcs_table.get(i - 1, j).unwrap()) {
        print_diff(lcs_table, lines1, lines2, i - 1, j);
        println!("< {}", lines1[i - 1]);
    } else {
        print!("");
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_read_file_lines() {
        let lines_result = read_file_lines("handout-a.txt");
        assert!(lines_result.is_ok());
        let lines = lines_result.unwrap();
        assert_eq!(lines.len(), 8);
        assert_eq!(
            lines[0],
            "This week's exercises will continue easing you into Rust and will feature some"
        );
    }

    #[test]
    fn test_lcs() {
        let mut expected = Grid::new(5, 4);
        expected.set(1, 1, 1).unwrap();
        expected.set(1, 2, 1).unwrap();
        expected.set(1, 3, 1).unwrap();
        expected.set(2, 1, 1).unwrap();
        expected.set(2, 2, 1).unwrap();
        expected.set(2, 3, 2).unwrap();
        expected.set(3, 1, 1).unwrap();
        expected.set(3, 2, 1).unwrap();
        expected.set(3, 3, 2).unwrap();
        expected.set(4, 1, 1).unwrap();
        expected.set(4, 2, 2).unwrap();
        expected.set(4, 3, 2).unwrap();

        println!("Expected:");
        expected.display();
        let result = lcs(
            &"abcd".chars().map(|c| c.to_string()).collect(),
            &"adb".chars().map(|c| c.to_string()).collect(),
        );
        println!("Got:");
        result.display();
        assert_eq!(result.size(), expected.size());
        for row in 0..expected.size().0 {
            for col in 0..expected.size().1 {
                assert_eq!(result.get(row, col), expected.get(row, col));
            }
        }
    }
}
