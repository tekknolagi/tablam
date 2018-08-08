//use std::ops;
use std::rc::Rc;

use super::types::*;

/// The ops in TablaM are columnar, and follow this pattern
/// [1, 2, 3] +  [1, 2, 3] = [2, 4, 6]
/// [1, 2, 3] +  1 = [1, 3, 4]
/// 1 + [1, 2, 3] = [1, 3, 4]
/// [1, 2, 3] +  [1, 2] = ERROR must be same length

// TODO: The operators follow this patterns:
// maps: ColumnExp & ColumnExp = Column (+ [1, 2] [2, 3] = [3, 6])
// reduce: ColumnExp = Column (+[1, 2] = 3)

/// Comparing 2 columns
#[inline]
fn _compare_both<'r, 's, T, F>(left:&'r [T], right:&'s [T], mut apply:F) -> Column
    where T: PartialEq,
          F: FnMut(&'r T, &'s T) -> bool
{
    let x:Vec<bool> = left.into_iter()
        .zip(right.into_iter())
        .map( |(x, y)| apply(x, y))
        .collect();
    Column::from(x)
}

/// Comparing a column with a scalar
fn _compare_col_scalar<T, F>(left:&[T], right:&T, mut apply:F) -> Column
    where T:PartialEq,
          F: FnMut(&T, &T) -> bool
{
    let x:Vec<bool> = left.into_iter()
        .map( | x | apply(x, right))
        .collect();
    Column::from(x)
}

/// Comparing 2 scalars
fn _compare_scalar_scalar<T, F>(left:&T, right:&T, mut apply:F) -> Column
    where T:PartialEq,
          F: FnMut(&T, &T) -> bool
{
    let x:Vec<bool> = vec!(apply(left, right));
    Column::from(x)
}

pub fn decode_both(left:&Column, right:&Column, op:Operator) -> Column {
    match (left, right) {
        (Column::I64(lhs), Column::I64(rhs))  => {
            let apply =
                match op {
                    Operator::Eq => PartialEq::eq,
                    Operator::NotEq => PartialEq::ne,
                    _ => panic!(" Operator {:?} not boolean", op)
                };

            _compare_both(lhs.as_slice(), rhs.as_slice(), apply)
        }
        (Column::UTF8(lhs), Column::UTF8(rhs)) => {
            let apply =
                match op {
                    Operator::Eq => PartialEq::eq,
                    Operator::NotEq => PartialEq::ne,
                    _ => panic!(" Operator {:?} not boolean", op)
                };

            _compare_both(lhs.as_slice(), rhs.as_slice(), apply)
        }
        (Column::BOOL(lhs), Column::BOOL(rhs))  =>{
            let apply =
                match op {
                    Operator::Eq => PartialEq::eq,
                    Operator::NotEq => PartialEq::ne,
                    _ => panic!(" Operator {:?} not boolean", op)
                };

            _compare_both(lhs.as_slice(), rhs.as_slice(), apply)
        }
        (Column::ROW(lhs), Column::ROW(rhs)) =>{
            let apply =
                match op {
                    Operator::Eq => PartialEq::eq,
                    Operator::NotEq => PartialEq::ne,
                    _ => panic!(" Operator {:?} not boolean", op)
                };

            _compare_both(lhs.as_slice(), rhs.as_slice(), apply)
        }
        (x , y) => panic!(" Incompatible {:?} and {:?}", x, y)
    }
}

pub fn equal_both(left:&Column, right:&Column) -> Column {
    decode_both(left, right, Operator::Eq)
}

pub fn equal_col_scalar(left:&Column, right:&Column) -> Column {
    decode_both(left, right, Operator::Eq)
}

pub fn not_equal_both(left:&Column, right:&Column) -> Column {
    decode_both(left, right, Operator::NotEq)
}

pub fn less_both(left:&Column, right:&Column) -> Column
{
    decode_both(left, right, Operator::Less)
}

pub fn greater_both(left:&Column, right:&Column) -> Column
{
    decode_both(left, right, Operator::Greater)
}

#[derive(Clone)]
pub struct CompareRel {
    rel:Rc<RelationRow>,
    pub op:Operator,
    pub left:ColumnExp,
    pub right:ColumnExp,
}

impl CompareRel {
    fn eq(rel:Rc<RelationRow>, left:ColumnExp, right:ColumnExp) -> Self {
        CompareRel {
            rel,
            op:Operator::Eq,
            left,
            right,
        }
    }
    fn noteq(rel:Rc<RelationRow>, left:ColumnExp, right:ColumnExp) -> Self {
        CompareRel {
            rel,
            op:Operator::NotEq,
            left,
            right,
        }
    }
}

pub fn select(of:Rc<RelationRow>, pick:&ColumnExp) -> Column {
    match pick {
        ColumnExp::Name(x) => of.col_named(x.as_str()),
        ColumnExp::Pos(x) => of.col(*x),
    }
}

fn filter(of: CompareRel) -> Column {
    let op = of.op;
    let rel = of.rel;

    let col1 = select(rel.clone(), &of.left);
    let col2 = select(rel.clone(), &of.right);

    match op {
        Operator::Eq        => equal_both(&col1, &col2),
        Operator::NotEq     => not_equal_both(&col1, &col2),
        Operator::Less      => less_both(&col1, &col2),
        Operator::Greater   => greater_both(&col1, &col2),
        _ => panic!(" Incompatible Operator {:?} in filter", op)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_nums1() -> Vec<i64> {
        vec![1, 2, 3]
    }

    fn make_nums2() -> Vec<i64> {
        vec![4, 2, 3]
    }

    fn col(pos:usize) -> ColumnExp {
        ColumnExp::Pos(pos)
    }

    fn make_rel1() -> Rc<Frame> {
        let nums1 = make_nums1();
        let nums2 = make_nums2();

        let col1 = Column::from(nums1);
        let col2 = Column::from(nums2);

        Rc::new(Frame::new(vec!(col1, col2)))
    }

    #[test]
    fn compare() {
        let nums1 = make_nums1();
        let f1 = make_rel1();
        let pick1 = ColumnExp::Name("col1".to_string());
        let pick2 = col(1);

        let col3 = select(f1.clone(), &pick1);
        let nums3:Vec<i64> = col3.as_slice().into();
        assert_eq!(nums1, nums3);

        let filter_eq = CompareRel::eq(f1.clone(), pick1, pick2);
        let filter_not_eq = CompareRel::noteq(f1.clone(), col(0), col(1));

        let result_eq:Vec<bool> = filter(filter_eq).as_slice().into();
        assert_eq!(result_eq, vec![false, true, true]);

        let result_not_eq:Vec<bool> = filter(filter_not_eq).as_slice().into();
        assert_eq!(result_not_eq, vec![true, false, false]);
    }


//    #[test]
//    fn math() {
//
//    }
}