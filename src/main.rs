use std::fmt::{Display, Formatter};
use std::str::FromStr;
use varisat::{CnfFormula, ExtendFormula, Solver, Var};

type Cell = Option<u8>;

#[derive(Clone)]
struct Grid {
    cells: [Cell; 81],
}

fn sudoku_formula() -> CnfFormula {
    let mut formula = CnfFormula::new();

    for y in 0..9 {
        for x in 0..9 {
            // Only one value per cell
            for a in 0..9 {
                let v_a = Var::from_index(y * 81 + x * 9 + a);
                for b in (a + 1)..9 {
                    let v_b = Var::from_index(y * 81 + x * 9 + b);
                    formula.add_clause(&[v_a.negative(), v_b.negative()]);
                }
            }

            // Each cell must contain at least one value.
            let select_at_least_one_clause = (0..9)
                .into_iter()
                .map(|v| Var::from_index(y * 81 + x * 9 + v).positive())
                .collect::<Vec<_>>();
            formula.add_clause(&select_at_least_one_clause);
        }
    }

    // For each row
    for y in 0..9 {
        for x in 0..9 {
            for d in 0..9 {
                for x2 in 0..9 {
                    if x != x2 {
                        formula.add_clause(&[
                            Var::from_index(y * 81 + x * 9 + d).negative(),
                            Var::from_index(y * 81 + x2 * 9 + d).negative(),
                        ])
                    }
                }
            }
        }
    }

    // For each column
    for x in 0..9 {
        for y in 0..9 {
            for d in 0..9 {
                for y2 in 0..9 {
                    if y != y2 {
                        formula.add_clause(&[
                            Var::from_index(y * 81 + x * 9 + d).negative(),
                            Var::from_index(y2 * 81 + x * 9 + d).negative(),
                        ])
                    }
                }
            }
        }
    }

    // For each block
    for block_idx in 0..9 {
        for i in 0..9 {
            for d in 0..9 {
                for i2 in 0..9 {
                    if i != i2 {
                        let x1 = i % 3;
                        let y1 = i / 3;
                        let x2 = i2 % 3;
                        let y2 = i2 / 3;
                        let bx = block_idx % 3;
                        let by = block_idx / 3;
                        formula.add_clause(&[
                            Var::from_index((by * 3 + y1) * 81 + (bx * 3 + x1) * 9 + d).negative(),
                            Var::from_index((by * 3 + y2) * 81 + (bx * 3 + x2) * 9 + d).negative(),
                        ])
                    }
                }
            }
        }
    }

    formula
}

impl Grid {
    pub fn get(&self, x: usize, y: usize) -> Cell {
        self.cells[y * 9 + x]
    }

    pub fn solve(mut self) -> Result<Grid, ()> {
        let mut solver = Solver::new();

        // Construct formula
        solver.add_formula(&sudoku_formula());

        // Add filled in values
        for y in 0..9 {
            for x in 0..9 {
                if let Some(d) = self.get(x, y) {
                    let v = Var::from_index(y * 81 + x * 9 + d as usize);
                    solver.add_clause(&[v.positive()]);
                }
            }
        }

        // Solve the damn thing
        if !solver.solve().unwrap() {
            return Err(());
        }

        // Get the values from the model
        let model = solver.model().unwrap();
        for var in model {
            if var.is_positive() {
                let digit = (var.index() % 9) as u8;
                let x = (var.index() % 81) / 9;
                let y = var.index() / 81;
                match std::mem::replace(&mut self.cells[y * 9 + x], Some(digit)) {
                    Some(prev) if prev != digit => {
                        unreachable!("decided something else!")
                    }
                    _ => {}
                }
            }
        }

        Ok(self)
    }
}

impl FromStr for Grid {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            cells: s
                .chars()
                .map(|c| match c {
                    '1'..='9' => c
                        .to_digit(10)
                        .map(|d| d as u8 - 1)
                        .ok_or_else(|| format!("invalid digit '{c}'"))
                        .map(Some),
                    ' ' => Ok(None),
                    _ => Err(format!("invalid character '{c}'")),
                })
                .collect::<Result<Vec<Cell>, _>>()?
                .try_into()
                .map_err(|e: Vec<Cell>| format!("failed to convert {:?} ({})", &e, e.len()))?,
        })
    }
}

impl Display for Grid {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for y in 0..9 {
            for x in 0..9 {
                match self.get(x, y) {
                    None => write!(f, " ")?,
                    Some(d) => write!(f, "{} ", d + 1)?,
                }
                if x < 8 && (x + 1) % 3 == 0 {
                    write!(f, "| ")?;
                }
            }
            writeln!(f)?;
            if y < 8 && (y + 1) % 3 == 0 {
                writeln!(f, "---------------------")?;
            }
        }
        Ok(())
    }
}

fn main() {}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_solve() {
        let grid: Grid =
            // "86   7           4 9 58 2    7   8 663   2  5 1  9  3  7   53  3    6     9    1 "
        // "1    7 9  3  2   8  96  5    53  9   1  8   26    4   3      1  41     7  7   3  "
        // "1                                                                               2"
        "8          36      7  9 2   5   7       457     1   3   1    68  85   1  9    4  "
                .parse()
                .unwrap();

        let grid = grid.solve().unwrap();

        println!("{grid}");
    }
}
