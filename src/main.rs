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

// NB: FOR NOW just a, b, @TODO add more input options

// @TODO think about whether we should add if already exists
fn push_sorted(vec: &mut Vec<usize>, new_int: usize) {
	match vec.binary_search(&new_int) {
		Ok(_) => {} // Already in sorted vec @NOTE we want this behavior, right?
		Err(i) => vec.insert(i, new_int),
	}
}

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
	// res[nfa0.len() - 1][2].push(nfa0.len());
	push_sorted(&mut res[nfa0.len() - 1][2], nfa0.len());

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
	// res[nfa0.len() - 1][2].push(final_idx);
	push_sorted(&mut res[nfa0.len() - 1][2], final_idx);
	// res[nfa0.len() + nfa1.len() - 1][2].push(final_idx);
	push_sorted(&mut res[nfa0.len() + nfa1.len() - 1][2], final_idx);

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
	// res[nfa.len() - 1][2].push(final_idx);
	push_sorted(&mut res[nfa.len() - 1][2], final_idx);

	// Insert start node with epislon move to nfa start and end
	res.insert(0, [vec![], vec![], vec![1, final_idx]]);

	// Connect end of nfa to start node
	// res[nfa.len()][2].push(0);
	push_sorted(&mut res[nfa.len()][2], 0);

	res
}

// NB: this should be trivial if always sorted
// fn check_eq(nfa0: &[[Vec<usize>; 3]], nfa1: &[[Vec<usize>; 3]]) -> bool {
// 	if nfa0.len() != nfa1.len() {
// 		return false;
// 	}

// 	for (row0, row1) in nfa0.iter().zip(nfa1.iter()) {
// 		if row0.len() != row1.len() {
// 			return false;
// 		}

// 		// assumes
// 	}
// 	true
// }

// @TODO tests, use trivial equality for comparing NFAs

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

	#[rustfmt::skip]
	assert_eq!(
		vec![
			[vec![1], vec![], vec![]],
			[vec![], vec![], vec![]]
		],
		a
	);

	// println!("Concatenating a and b:");
	let a_concat_b = concat(&a, &b);
	// print_nfa(&a_concat_b);
	#[rustfmt::skip]
	assert_eq!(
		vec![
			[vec![1], vec![],vec! []],
			[vec![], vec![], vec![2]],
			[vec![], vec![3], vec![]],
			[vec![], vec![], vec![]],
		],
		a_concat_b
	);

	// println!("Concatenating ab and b:");
	// print_nfa(&concat(&a_concat_b, &b));

	// println!("=======================");
	// println!("Unioning a and b:");
	let a_union_b = union(&a, &b);
	print_nfa(&a_union_b);
	#[rustfmt::skip]
	assert_eq!(
		vec![
			[vec![], vec![], vec![1, 3]],
			[vec![2], vec![], vec![]],
			[vec![], vec![], vec![5]],
			[vec![], vec![2],vec! []],
			[vec![], vec![], vec![5]],
			[vec![], vec![], vec![]],
		],
		a_union_b
	);

	// println!("=======================");
	// println!("A*:");
	let a_star = star(&a);
	// print_nfa(&a_star);
	#[rustfmt::skip]
	assert_eq!(
		vec![
			[vec![], vec![], vec![1, 3]],
			[vec![2],vec![], vec![]],
			[vec![], vec![], vec![0, 3]],
			[vec![], vec![], vec![]],
		],
		a_star
	);

	// println!("=======================");
	// println!("(A|B)*");
	let nfa = star(&union(&a, &b));
	// print_nfa(&nfa);
	#[rustfmt::skip]
	assert_eq!(
		vec![
			[vec![], vec![], vec![1, 7]],
			[vec![], vec![], vec![2, 4]],
			[vec![3],vec![], vec![]],
			[vec![], vec![], vec![6]],
			[vec![], vec![3], vec![]],
			[vec![], vec![], vec![6]],
			[vec![], vec![], vec![0, 7]],
			[vec![], vec![], vec![]],
		],
		nfa
	);
}
