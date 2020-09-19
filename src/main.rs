// NFA implementation
// Hoping that it's easy to translate regex to these

// NB:
// a : O a-> ◎
// ε : O ε-> ◎

// NFA table spec:
// Rows = states
// Columns = all possible inputs plus one for epsilon

// NB: later, change nfa to struct, vec with inputs, table with vec of
// vec (should this be Option<vec>? Are empty rows common?) of Option(vec)
// i.e. Vec<Vec<Option<Vec>>>
// @TODO implement fmt::Debug (better than fmt::Display) trait for NFA type
// later

// NFA from
// https://www.geeksforgeeks.org/program-implement-nfa-epsilon-move-dfa-conversion/

// nfa.push([vec![5, 2], vec![], vec![1, 5]]);
// nfa.push([vec![], vec![2], vec![]]);
// nfa.push([vec![], vec![], vec![3]]);
// nfa.push([vec![4], vec![], vec![1, 5]]);
// nfa.push([vec![], vec![], vec![]]);

// NB: FOR NOW just a, b, @TODO add more input options

fn print_nfa(nfa: &[[Vec<usize>; 3]]) {
	for (i, row) in nfa.iter().enumerate() {
		println!("{} {:?}", i, row);
	}
}

fn concat(nfa0: &[[Vec<usize>; 3]], nfa1: &[[Vec<usize>; 3]]) -> Vec<[Vec<usize>; 3]> {
	// NB when working with larger alphabets, update alphabets for each here first
	// NB THIS DOESNT NEED TO BE OPTIMIZED IT JUST NEEDS TO WORK, optimize DFA
	// matching at runtime

	let mut res = nfa0.clone().to_vec();

	// This could be done with one pass but I'm lazy

	for row in nfa1.iter() {
		res.push((*row).clone());
	}

	for row in res.iter_mut().skip(nfa0.len()) {
		for state_list in row.iter_mut() {
			for state in state_list.iter_mut() {
				*state += nfa0.len();
			}
		}
	}

	// Connect final state of nfa0 to start state of nfa1
	res[nfa0.len() - 1][2].push(nfa0.len());

	res
}

fn union(nfa0: &[[Vec<usize>; 3]], nfa1: &[[Vec<usize>; 3]]) -> Vec<[Vec<usize>; 3]> {
	let mut res = nfa0.clone().to_vec();

	for row in nfa1.iter() {
		res.push((*row).clone());
	}

	// Update references before we insert new node
	for row in res.iter_mut() {
		for state_list in row.iter_mut() {
			for state in state_list.iter_mut() {
				*state += 1;
			}
		}
	}

	// new final node
	res.push([vec![], vec![], vec![]]);

	// update nfa0 and nfa1 final nodes with epsilon moves to final node
	// NB: res.len() will be idx of final node after next insert
	let final_idx = res.len();
	res[nfa0.len() - 1][2].push(final_idx);
	res[nfa0.len() + nfa1.len() - 1][2].push(final_idx);

	// node 0 goes to nfa0 and nfa1 start by epsilon
	res.insert(0, [vec![], vec![], vec![1, nfa0.len() + 1]]);

	res
}

fn star(nfa: &[[Vec<usize>; 3]]) -> Vec<[Vec<usize>; 3]> {
	let mut res = nfa.clone().to_vec();

	// Update references: there will be 1 insertion before nfa
	for row in res.iter_mut() {
		for state_list in row.iter_mut() {
			for state in state_list.iter_mut() {
				*state += 1;
			}
		}
	}

	// new final node
	res.push([vec![], vec![], vec![]]);

	// epsilon move from nfa end to final state
	// NB: res.len() will be idx of final node after next insert
	let final_idx = res.len();
	res[nfa.len() - 1][2].push(final_idx);

	// Insert start node with epislon move to nfa start and end
	res.insert(0, [vec![], vec![], vec![1, final_idx]]);

	// Connect end of nfa to start node
	res[nfa.len()][2].push(0);

	res
}

fn main() {
	// NB: for now, start state is just first state and last state is just final
	// state

	// a : O 0-> ◎
	let mut a = Vec::new();
	a.push([vec![1], vec![], vec![]]);
	a.push([vec![], vec![], vec![]]);

	let mut b = Vec::new();
	b.push([vec![], vec![1], vec![]]);
	b.push([vec![], vec![], vec![]]);

	let mut epsilon = Vec::new();
	epsilon.push([vec![], vec![], vec![1]]);
	epsilon.push([vec![], vec![], vec![]]);

	print_nfa(&a);
	println!("");
	print_nfa(&b);
	println!("");
	print_nfa(&epsilon);

	println!("Concatenating a and b:");
	let a_concat_b = concat(&a, &b);
	print_nfa(&a_concat_b);
	println!("Concatenating ab and b:");
	print_nfa(&concat(&a_concat_b, &b));

	println!("=======================");
	println!("Unioning a and b:");
	let a_union_b = union(&a, &b);
	print_nfa(&a_union_b);

	println!("=======================");
	println!("A*:");
	let a_star = star(&a);
	print_nfa(&a_star);

	println!("=======================");
	println!("(A|B)*");
	let nfa = star(&union(&a, &b));
	print_nfa(&nfa);
}
