use std::fmt;
use std::time::{Duration, Instant};

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

// NB: would something like array2d be more efficient than nested vecs?

// NB: why do we need Debug, for PartialEq?
// And can we remove it in the future to implement our own?

// @TODO optimization: change 2 dimensional Vec<Vec<T>>s to 1 dimensional Vec<T>
// @TODO optimization: change as many sorted Vecs to HashSets as possible
// @TODO optimization: don't create so many useless rows from concat, union, and
// star operations on NFAs

#[derive(Clone, Default, PartialEq)]
struct NFA {
	inputs: Vec<char>,
	table: Vec<Vec<Vec<usize>>>,
}

impl fmt::Debug for NFA {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		writeln!(f, "  {:?}", self.inputs)?;

		for (i, row) in self.table.iter().enumerate() {
			writeln!(f, "{} {:?}", i, row)?;
		}

		Ok(())
	}
}

impl NFA {
	// @NOTE why can an impl method not contain a reference to self?

	fn eps_clos(&self, base_state: usize) -> Vec<usize> {
		let mut res = Vec::with_capacity(self.table.len());
		res.push(base_state);

		let e_moves = self.table[base_state].last().unwrap();

		// Base case
		if *e_moves == vec![] || *e_moves == vec![base_state] {
			return res;
		}

		// @NOTE would be much more efficient (mut slices vs passing vecs as
		// return vals) with iteration instead of recursion
		for &state in e_moves {
			match res.binary_search(&state) {
				Ok(_) => {}
				Err(i) => {
					res.insert(i, state);

					let e_clos = self.eps_clos(state);

					// Union of e_clos and res
					// @NOTE this could be made more efficient by implementing a
					// "merge" routine for sorted vecs, but size prob never
					// large enough to get any performance gains
					for state in e_clos {
						push_sorted(&mut res, state);
					}
				}
			}
		}

		res
	}

	// AUXILIARY FUNCTION:
	// Populates eps_closures with the epsilon closures of i, and populates DFA
	// with states corresponding to the new epsilon closures. Returns the
	// indices of inserted epsilon closures
	fn eps_clos_from_eps_clos(
		&self,
		eps_closures: &mut Vec<Vec<usize>>,
		dfa: &mut DFA,
		i: usize,
	) -> Vec<usize> {
		let mut res = vec![];

		// I hate rust sometimes -- cf clone
		// Not bad enough here to cause a real performance impact, but...
		// Can't push to eps_closures on line 117 because we're iterating over
		// eps_closures[0]
		// @TODO see how to do this in unsafe rust?
		for &from_state in eps_closures[i].clone().iter() {
			for input_idx in 0..self.inputs.len() {
				let non_eps_moves_iter =
					self.table[from_state][input_idx].iter();

				for &to_state in non_eps_moves_iter {
					let eps_clos = self.eps_clos(to_state);

					let dfa_state: usize;

					// This is inefficient
					// @OPTIMIZE? if possible? if necessary?
					// Store these in some sort of sorted way?
					// if !eps_closures.contains(&eps_clos) {
					// 	eps_closures.push(eps_clos);
					// }
					// @LEARN why is
					// `match eps_closures.iter().position(|&e| e == eps_clos)`
					// not ok (cannot move out of a shared reference)
					// but
					// `match eps_closures.iter().position(|e| *e == eps_clos)`
					// is?
					// NB: position returns index of eps_clos in eps_closures

					// NB: don't think I should need to copy eps_clos here, but
					// rust is dumb and wants to prevent use after move
					match eps_closures.iter().position(|e| *e == eps_clos) {
						Some(i) => dfa_state = i,
						None => {
							eps_closures.push(eps_clos.clone());
							dfa_state = eps_closures.len() - 1;
							dfa.table.push(vec![None; dfa.inputs.len()]);

							res.push(dfa_state);
						}
					}

					dfa.table[i][input_idx] = Some(dfa_state);

					match eps_clos.last() {
						Some(&nfa_state) => {
							let final_nfa_state = self.table.len() - 1;
							if nfa_state == final_nfa_state {
								push_sorted(&mut dfa.final_states, dfa_state);
							}
						}
						None => {}
					}
				}
			}
		}

		res
	}

	fn to_dfa(&self) -> DFA {
		let mut dfa = DFA {
			// @LEARN what is the difference between copy() and clone() here?
			inputs: self.inputs.clone(),
			..Default::default()
		};

		dfa.table.push(vec![None; dfa.inputs.len()]);

		let mut eps_closures: Vec<Vec<usize>> = vec![];
		eps_closures.push(self.eps_clos(0));

		// final_state code not called for state 0
		// @TODO cleanup?
		match eps_closures[0].last() {
			// @NOTE Written using `push_sorted` so I can change to HashSet more
			// easily later if necessary
			Some(_) => push_sorted(&mut dfa.final_states, 0),
			None => {}
		}

		let mut stack = vec![0usize];

		// @NOTE UNTESTED for more complex NFAs
		while !stack.is_empty() {
			let new_dfa_states = self.eps_clos_from_eps_clos(
				&mut eps_closures,
				&mut dfa,
				stack.pop().unwrap(),
			);
			stack.extend(&new_dfa_states);
		}

		dfa
	}
}

#[derive(Default)]
struct DFA {
	inputs: Vec<char>,
	final_states: Vec<usize>,
	// @TODO replace Vec<Option<usize>> with RC or other multiple-reference
	// pointer value? Only if space is an issue, which it doesn't appear to be
	table: Vec<Vec<Option<usize>>>,
}

impl DFA {
	// NB: only matches at the start of the string
	// NB: only matches the entire string
	// @TODO return matches on all substrings of input (push index to some vec
	// every time we have a match)
	fn mtch(&self, input: &str) -> bool {
		let mut state = 0;
		let mut matching = self.final_states.binary_search(&state).is_ok();
		for chr in input.chars() {
			// Super inefficient. Need to start using hash sets and hash tables
			let input_idx = self.inputs.binary_search(&chr);

			match input_idx {
				Ok(input_idx) => match self.table[state][input_idx] {
					Some(new_state) => state = new_state,
					None => return false,
				},
				// input not in nfa!
				// @TODO maybe return error?
				Err(_) => return false,
			}

			matching = self.final_states.binary_search(&state).is_ok();
		}

		matching
	}
}

impl fmt::Debug for DFA {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		writeln!(f, "{:?}", self.inputs)?;

		for (i, row) in self.table.iter().enumerate() {
			writeln!(f, "{} {:?}", i, row)?;
		}

		write!(f, "final: {:?}", self.final_states)?;

		Ok(())
	}
}

fn push_sorted<T>(vec: &mut Vec<T>, new_int: T)
where
	T: Ord,
{
	match vec.binary_search(&new_int) {
		// Don't do anything, already in sorted vec @NOTE we want this behavior,
		// right?
		Ok(_) => {}
		Err(i) => vec.insert(i, new_int),
	}
}

// @TODO use to optimize union function
fn combine_vecs<T>(vec0: &[T], vec1: &[T]) -> Vec<T>
where
	T: Ord,
	T: Clone,
{
	let mut res = Vec::with_capacity(std::cmp::max(vec0.len(), vec1.len()));
	// NB: this could be done more efficiently with a merge subroutine like in
	// mergesort
	// @TODO implement?
	for val0 in vec0.iter() {
		res.push(val0.clone());
	}
	for val1 in vec1.iter() {
		push_sorted(&mut res, val1.clone());
	}
	res
}

// @TODO use to optimize union function
fn combine_rows<T>(row0: &[Vec<T>], row1: &[Vec<T>]) -> Vec<Vec<T>>
where
	T: Ord,
	T: Clone,
{
	let mut res = Vec::with_capacity(row0.len());

	// NOTE: if row0, row1 not equal length, silently fails
	for (v0, v1) in row0.iter().zip(row1.iter()) {
		res.push(combine_vecs(v0, v1));
	}

	res
}

// @NOTE: alters nfa0 and nfa1
fn join_alphabets(nfa0: &mut NFA, nfa1: &mut NFA) {
	// let mut joint_inputs = nfa0.inputs.clone();

	// @NOTE could be done better, join alphabets then one loop. But I'm lazy.
	// Pretty sure to do it that way you'd have to use an iter().enumerate()
	for chr in nfa1.inputs.iter() {
		match nfa0.inputs.binary_search(chr) {
			Ok(_) => {} // Don't need to do anything if already there
			Err(i) => {
				nfa0.inputs.insert(i, *chr);
				for row in nfa0.table.iter_mut() {
					row.insert(i, vec![]);
				}
			}
		}
	}

	// Same as above, but nfa0 and nfa1 swapped
	for chr in nfa0.inputs.iter() {
		match nfa1.inputs.binary_search(chr) {
			Ok(_) => {} // Don't need to do anything if already there
			Err(i) => {
				nfa1.inputs.insert(i, *chr);
				for row in nfa1.table.iter_mut() {
					row.insert(i, vec![]);
				}
			}
		}
	}
}

fn concat(nfa0: &NFA, nfa1: &NFA) -> NFA {
	// NB when working with larger alphabets, update alphabets for each here
	// first NB THIS DOESNT NEED TO BE OPTIMIZED IT JUST NEEDS TO WORK, optimize
	// DFA matching at runtime

	// Inefficient but blah. Don't want to mutate params
	let mut nfa0 = nfa0.clone();
	let mut nfa1 = nfa1.clone();
	join_alphabets(&mut nfa0, &mut nfa1);

	let mut res = nfa0.clone();

	// This could be done with one pass but I'm lazy

	for row in nfa1.table.iter() {
		res.table.push((*row).clone());
	}

	for row in res.table.iter_mut().skip(nfa0.table.len()) {
		for state_list in row.iter_mut() {
			for state in state_list.iter_mut() {
				*state += nfa0.table.len();
			}
		}
	}

	// Connect final state of nfa0 to start state of nfa1
	push_sorted(
		&mut res.table[nfa0.table.len() - 1].last_mut().unwrap(),
		nfa0.table.len(),
	);

	res
}

fn union(nfa0: &NFA, nfa1: &NFA) -> NFA {
	// Inefficient but blah. Don't want to mutate params
	let mut nfa0 = nfa0.clone();
	let mut nfa1 = nfa1.clone();
	join_alphabets(&mut nfa0, &mut nfa1);

	let mut res = nfa0.clone();

	// Update references in nfa1 before we insert
	for row in nfa1.table.iter_mut() {
		for state_list in row.iter_mut() {
			for state in state_list.iter_mut() {
				*state += nfa0.table.len();
			}
		}
	}

	for row in nfa1.table.iter() {
		res.table.push((*row).clone());
	}

	// Update references before we insert starting node node
	for row in res.table.iter_mut() {
		for state_list in row.iter_mut() {
			for state in state_list.iter_mut() {
				*state += 1;
			}
		}
	}

	// new final node
	let mut empty_row = Vec::with_capacity(res.inputs.len() + 1);
	// +1 for epsilon
	for _ in 0..res.inputs.len() + 1 {
		empty_row.push(vec![]);
	}
	res.table.push(empty_row);

	// update nfa0 and nfa1 final nodes with epsilon moves to final node
	// NB: res.table.len() will be idx of final node after next insert
	let final_idx = res.table.len();

	push_sorted(
		&mut res.table[nfa0.table.len() - 1].last_mut().unwrap(),
		final_idx,
	);

	push_sorted(
		&mut res.table[nfa0.table.len() + nfa1.table.len() - 1]
			.last_mut()
			.unwrap(),
		final_idx,
	);

	// node 0 goes to nfa0 and nfa1 start by epsilon
	let mut start_row = Vec::with_capacity(res.inputs.len() + 1);
	for _ in 0..res.inputs.len() {
		start_row.push(vec![]);
	}
	start_row.push(vec![1, nfa0.table.len() + 1]);
	res.table.insert(0, start_row);

	res
}

// @TODO think about .unwrap() and edge cases (empty NFA?)
fn star(nfa: &NFA) -> NFA {
	let mut res = nfa.clone();

	// Update references: there will be 1 insertion before nfa
	for row in res.table.iter_mut() {
		for state_list in row.iter_mut() {
			for state in state_list.iter_mut() {
				*state += 1;
			}
		}
	}

	// new final node
	let mut empty_row = Vec::with_capacity(res.inputs.len() + 1);
	// +1 for epsilon
	for _ in 0..res.inputs.len() + 1 {
		empty_row.push(vec![]);
	}
	res.table.push(empty_row);

	// epsilon move from nfa end to final state
	// NB: res.table.len() will be idx of final node after next insert
	let final_idx = res.table.len();
	push_sorted(
		&mut res.table[nfa.table.len() - 1].last_mut().unwrap(),
		final_idx,
	);

	// Insert start node with epislon move to nfa start and end

	let mut start_row = Vec::with_capacity(res.inputs.len() + 1);
	for _ in 0..res.inputs.len() {
		start_row.push(vec![]);
	}
	start_row.push(vec![1, final_idx]);
	res.table.insert(0, start_row);

	// Connect end of nfa to start node
	push_sorted(&mut res.table[nfa.table.len()].last_mut().unwrap(), 0);

	res
}

fn plus(nfa: &NFA) -> NFA {
	concat(&nfa, &star(&nfa))
}

fn opt(nfa: &NFA) -> NFA {
	let eps = NFA {
		..Default::default()
	};
	union(&nfa, &eps)
}

fn single_char_nfa(c: char) -> NFA {
	let inputs = vec![c];
	let mut table = Vec::new();
	table.push(vec![vec![1], vec![]]);
	table.push(vec![vec![], vec![]]);

	NFA { inputs, table }
}

// @TODO use rust tests
fn run_nfa_tests() {
	let a = single_char_nfa('a');
	let b = single_char_nfa('b');
	let c = single_char_nfa('c');
	let d = single_char_nfa('d');
	let e = single_char_nfa('e');
	let f = single_char_nfa('f');

	let a_concat_b = concat(&a, &b);

	assert_eq!(
		vec![
			vec![vec![1], vec![], vec![]],
			vec![vec![], vec![], vec![2]],
			vec![vec![], vec![3], vec![]],
			vec![vec![], vec![], vec![]],
		],
		a_concat_b.table
	);

	let a_union_b = union(&a, &b);

	assert_eq!(
		vec![
			vec![vec![], vec![], vec![1, 3]],
			vec![vec![2], vec![], vec![]],
			vec![vec![], vec![], vec![5]],
			vec![vec![], vec![4], vec![]],
			vec![vec![], vec![], vec![5]],
			vec![vec![], vec![], vec![]],
		],
		a_union_b.table
	);

	let a_star = star(&a);

	assert_eq!(
		vec![
			vec![vec![], vec![1, 3]],
			vec![vec![2], vec![]],
			vec![vec![], vec![0, 3]],
			vec![vec![], vec![]],
		],
		a_star.table
	);

	let a_d = concat(&a, &d);
	let b_e = concat(&b, &e);
	let c_f = concat(&c, &f);

	let a_thru_f_concat = concat(&concat(&a_d, &b_e), &c_f);

	// This implicitly tests `join_alphabets`
	assert_eq!(
		NFA {
			inputs: vec!['a', 'b', 'c', 'd', 'e', 'f'],
			table: vec![
				vec![vec![1], vec![], vec![], vec![], vec![], vec![], vec![]],
				vec![vec![], vec![], vec![], vec![], vec![], vec![], vec![2]],
				vec![vec![], vec![], vec![], vec![3], vec![], vec![], vec![]],
				vec![vec![], vec![], vec![], vec![], vec![], vec![], vec![4]],
				vec![vec![], vec![5], vec![], vec![], vec![], vec![], vec![]],
				vec![vec![], vec![], vec![], vec![], vec![], vec![], vec![6]],
				vec![vec![], vec![], vec![], vec![], vec![7], vec![], vec![]],
				vec![vec![], vec![], vec![], vec![], vec![], vec![], vec![8]],
				vec![vec![], vec![], vec![9], vec![], vec![], vec![], vec![]],
				vec![vec![], vec![], vec![], vec![], vec![], vec![], vec![10]],
				vec![vec![], vec![], vec![], vec![], vec![], vec![11], vec![]],
				vec![vec![], vec![], vec![], vec![], vec![], vec![], vec![]],
			]
		},
		a_thru_f_concat
	);
}

fn union_char_range(start_char: char, end_char: char) -> NFA {
	let mut nfa = NFA {
		inputs: (start_char..=end_char).collect::<Vec<char>>(),
		table: Vec::with_capacity(2),
	};

	let num_chars = end_char as usize - start_char as usize + 1;

	nfa.table.push(Vec::with_capacity(num_chars));
	nfa.table.push(Vec::with_capacity(num_chars));

	// Excludes epsilon column
	for _ in 0..num_chars {
		nfa.table.first_mut().unwrap().push(vec![1]);
		nfa.table.last_mut().unwrap().push(vec![]);
	}
	// Epsilon column
	nfa.table.first_mut().unwrap().push(vec![]);
	nfa.table.last_mut().unwrap().push(vec![]);

	nfa
}

// @TODO make proper tests
fn test_dfa_conversion() -> (Duration, Duration, Duration) {
	// CREATION
	// ---------------------------------------------------------
	let now = Instant::now();

	let dig = &union_char_range('0', '9');

	// Make regex: (\d+((E(\+|-)?\d+)|(\.\d+))?)

	let int = plus(dig);

	let e = single_char_nfa('E');
	let pls = single_char_nfa('+');
	let min = single_char_nfa('-');
	let mut exp = concat(&e, &opt(&union(&pls, &min)));
	exp = concat(&exp, &plus(dig));

	let num_lit_nfa = concat(&int, &opt(&exp));

	let creation_time = Instant::now() - now;

	// CONVERSION
	// ---------------------------------------------------------

	let now = Instant::now();
	let num_lit_dfa = num_lit_nfa.to_dfa();
	let conversion_time = Instant::now() - now;

	// MATCHING
	// ---------------------------------------------------------

	let now = Instant::now();
	assert!(num_lit_dfa.mtch("10000"));
	assert!(num_lit_dfa.mtch("0238974982734"));
	assert!(num_lit_dfa.mtch("2E23"));
	assert!(num_lit_dfa.mtch("2E-23"));
	assert!(num_lit_dfa.mtch("43E+43"));
	assert!(!num_lit_dfa.mtch("43E"));
	let match_time = Instant::now() - now;

	(creation_time, conversion_time, match_time)
}

fn main() {
	run_nfa_tests();
	println!("All NFA tests passed :)");

	println!("Number literal nfa:");
	let (creation_time, conversion_time, match_time) = test_dfa_conversion();
	println!(
		"Creation: {:?} Conversion: {:?}, Matches: {:?}",
		creation_time, conversion_time, match_time
	);
}
