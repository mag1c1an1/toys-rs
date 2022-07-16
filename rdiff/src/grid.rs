pub struct Grid {
    num_rows: usize,
    num_cols: usize,
    elems: Vec<usize>,
}

impl Grid {
    pub fn new(num_rows: usize, num_cols: usize) -> Grid {
        Grid {
            num_rows,
            num_cols,
            elems: vec![0; num_rows * num_cols],
        }
    }
    pub fn size(&self) -> (usize, usize) {
        (self.num_rows, self.num_cols)
    }

    pub fn get(&self, row: usize, col: usize) -> Option<usize> {
        if row<self.num_rows && col < self.num_cols{
             Some(self.elems[row * self.num_cols + col]) 
        }else {
            None
        }
    }

    pub fn set(&mut self, row: usize, col: usize,value:usize) -> Result<(), &'static str> {
        if row<self.num_rows && col < self.num_cols{
            self.elems[row * self.num_cols + col ] = value;
            Ok(())
        }else {
            Err("Grid::set : Out of Bound!")
        }
    }

    pub fn display(&self) {
        for row in 0..self.num_rows {
            let mut line  = String::new();
            for col in 0..self.num_cols {
                line.push_str(&format!("{}, ",self.get(row,col).unwrap()));
            }
            println!("{line}")
        }
    }

    pub fn clear (&mut self) {
        for i in self.elems.iter_mut() {
            *i = 0;
        }
    }
}

#[cfg(test)]
mod test{
    use super::*;

    #[test]
    fn test_grid() {
       let n_rows = 4; 
       let n_cols = 3;
       let mut grid = Grid::new(n_rows,n_cols);

       for r in 0..n_rows{
        for c in 0..n_cols {
            assert!(
                grid.set(r,c, r*n_cols + c).is_ok(),
                "Grid::set returned Err even though the provided bounds are valid!"
            );
        }
       }

       println!("Grid contents:");
       grid.display();

       for r in 0..n_rows {
        for c in 0..n_cols {
            assert!(
                grid.get(r,c).is_some(),
                "Grid::get returned Err even thought the provided bounds are valid!"
            );
            assert_eq!(grid.get(r, c).unwrap(),r*n_cols+c);
        }
       }
    }
}