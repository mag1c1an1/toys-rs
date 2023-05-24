use std::{println, rc::Rc};

#[derive(Clone, Debug)]
struct Tree {
    value: u32,
    a: Option<Rc<Tree>>,
    b: Option<Rc<Tree>>,
}

fn modify_tree<F>(tree: &mut Rc<Tree>, f: &F)
where
    F: Fn(&mut Rc<Tree>) -> Option<&mut Tree>,
{
    if let Some(tree_mut) = f(tree) {
        if let Some(a_mut) = tree_mut.a.as_mut() {
            modify_tree(a_mut, f);
        }
        if let Some(b_mut) = tree_mut.b.as_mut() {
            modify_tree(b_mut, f);
        }
    }
}

fn main() {
    let origin_tree = Rc::new(Tree {
        value: 4,
        a: Some(Rc::new(Tree {
            value: 1,
            a: None,
            b: None,
        })),
        b: Some(Rc::new(Tree {
            value: 3,
            a: None,
            b: None,
        })),
    });
    let mut new_tree = origin_tree.clone();

    modify_tree(&mut new_tree, &|node: &mut Rc<Tree>| -> Option<&mut Tree> {
        if node.value > 2 {
            let node_mut = Rc::make_mut(node);
            node_mut.value -= 1;
            Some(node_mut)
        } else {
            None
        }
    });

    println!("{:#?}", origin_tree);
    println!("{:#?}", new_tree);
}
