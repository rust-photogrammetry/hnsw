use generic_array::{typenum, ArrayLength};
use gnuplot::*;
use hnsw::*;
use rand::distributions::{Bernoulli, Standard};
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64;
use std::cell::RefCell;
use structopt::StructOpt;

/// This is the probability each bit of an inlier will be different.
/// This comes from "Online Nearest Neighbor Search in Hamming Space"
/// in figure 2a, where 1-NN has an average search radius of 11 for
/// 128-bit features. I make the assumption here that the inliers
/// exist on a binomial distribution over 128 choices centered at
/// 11, which is consistent with the inlier statistics found in the paper
/// "ORB: an efficient alternative to SIFT or SURF".
const BIT_DIFF_PROBABILITY_OF_INLIER: f64 = 0.0859;

#[derive(Debug, StructOpt)]
#[structopt(name = "recall", about = "Generates recall graphs for HNSW")]
struct Opt {
	/// The value of M to use.
	///
	/// This can only be between 1 and 64 inclusive. M0 is set to 2 * M.
	#[structopt(short = "m", long = "max_edges", default_value = "12")]
	m: usize,
	/// The dataset size to test on.
	#[structopt(short = "s", long = "size", default_value = "65536")]
	size: usize,
	/// Total number of inlier search bitstrings.
	///
	/// The higher this is, the better the quality of the output data and statistics.
	#[structopt(short = "i", long = "inliers", default_value = "1000")]
	inliers: usize,
	/// The beginning ef value.
	#[structopt(short = "b", long = "beginning_ef", default_value = "1")]
	beginning_ef: usize,
	/// The ending ef value.
	#[structopt(short = "e", long = "ending_ef", default_value = "32")]
	ending_ef: usize,
}

fn process<M: ArrayLength<u32>, M0: ArrayLength<u32>>(opt: &Opt) -> (Vec<f64>, Vec<f64>) {
	let rng = Pcg64::from_seed([5; 32]);

	eprintln!("Generating {} random bitstrings...", opt.size);
	let search_space = rng
		.sample_iter(&Standard)
		.take(opt.size)
		.collect::<Vec<u128>>();
	eprintln!("Done.");

	let mut rng = Pcg64::from_seed([6; 32]);

	eprintln!(
		"Generating {} random inliers with probability of bit mutation of {}...",
		opt.inliers, BIT_DIFF_PROBABILITY_OF_INLIER
	);
	let bernoulli = Bernoulli::new(BIT_DIFF_PROBABILITY_OF_INLIER).unwrap();
	let query_strings = search_space
		.choose_multiple(&mut rng, opt.inliers)
		.map(|&feature| {
			let mut feature = feature;
			for bit in 0..128 {
				let choice: bool = rng.sample(&bernoulli);
				feature ^= (choice as u128) << bit;
			}
			feature
		})
		.collect::<Vec<u128>>();
	eprintln!("Done.");

	eprintln!(
		"Computing the correct nearest neighbor distance for all {} inliers...",
		opt.inliers
	);
	let correct_distances: Vec<u32> = query_strings
		.iter()
		.cloned()
		.map(|feature| {
			search_space
				.iter()
				.cloned()
				.map(|n| (feature ^ n).count_ones())
				.min()
				.unwrap()
		})
		.collect();
	eprintln!("Done.");

	eprintln!("Generating HNSW...");
	let mut hnsw: HNSW<M, M0> = HNSW::new();
	let mut searcher = Searcher::default();
	for &feature in &search_space {
		hnsw.insert(feature, &mut searcher);
	}
	eprintln!("Done.");

	eprintln!("Computing recall graph...");
	let efs = opt.beginning_ef..=opt.ending_ef;
	let state = RefCell::new((searcher, query_strings.iter().cloned().enumerate().cycle()));
	let (recalls, times): (Vec<f64>, Vec<f64>) = efs
		.map(|ef| {
			let correct = RefCell::new(0usize);
			let stats = easybench::bench(|| {
				let mut refmut = state.borrow_mut();
				let (searcher, query) = &mut *refmut;

				let mut dest = [0; 1];
				let (ix, query_feature) = query.next().unwrap();
				if (search_space[hnsw.nearest(query_feature, ef, searcher, &mut dest)[0] as usize]
					^ query_feature)
					.count_ones() == correct_distances[ix]
				{
					*correct.borrow_mut() += 1;
				}
			});
			(stats, correct.into_inner())
		})
		.fold(
			(vec![], vec![]),
			|(mut recalls, mut times), (stats, correct)| {
				times.push((stats.ns_per_iter * 0.1f64.powi(9)).recip());
				recalls.push(correct as f64 / stats.iterations as f64);
				(recalls, times)
			},
		);
	eprintln!("Done.");

	(recalls, times)
}

fn main() {
	let opt = Opt::from_args();

	let (recalls, times) = {
		use typenum::*;
		match opt.m {
			1 => process::<U1, U2>(&opt),
			2 => process::<U2, U4>(&opt),
			3 => process::<U3, U6>(&opt),
			4 => process::<U4, U8>(&opt),
			5 => process::<U5, U10>(&opt),
			6 => process::<U6, U12>(&opt),
			7 => process::<U7, U14>(&opt),
			8 => process::<U8, U16>(&opt),
			9 => process::<U9, U18>(&opt),
			10 => process::<U10, U20>(&opt),
			11 => process::<U11, U22>(&opt),
			12 => process::<U12, U24>(&opt),
			13 => process::<U13, U26>(&opt),
			14 => process::<U14, U28>(&opt),
			15 => process::<U15, U30>(&opt),
			16 => process::<U16, U32>(&opt),
			17 => process::<U17, U34>(&opt),
			18 => process::<U18, U36>(&opt),
			19 => process::<U19, U38>(&opt),
			20 => process::<U20, U40>(&opt),
			21 => process::<U21, U42>(&opt),
			22 => process::<U22, U44>(&opt),
			23 => process::<U23, U46>(&opt),
			24 => process::<U24, U48>(&opt),
			25 => process::<U25, U50>(&opt),
			26 => process::<U26, U52>(&opt),
			27 => process::<U27, U54>(&opt),
			28 => process::<U28, U56>(&opt),
			29 => process::<U29, U58>(&opt),
			30 => process::<U30, U60>(&opt),
			31 => process::<U31, U62>(&opt),
			32 => process::<U32, U64>(&opt),
			33 => process::<U33, U66>(&opt),
			34 => process::<U34, U68>(&opt),
			35 => process::<U35, U70>(&opt),
			36 => process::<U36, U72>(&opt),
			37 => process::<U37, U74>(&opt),
			38 => process::<U38, U76>(&opt),
			39 => process::<U39, U78>(&opt),
			40 => process::<U40, U80>(&opt),
			41 => process::<U41, U82>(&opt),
			42 => process::<U42, U84>(&opt),
			43 => process::<U43, U86>(&opt),
			44 => process::<U44, U88>(&opt),
			45 => process::<U45, U90>(&opt),
			46 => process::<U46, U92>(&opt),
			47 => process::<U47, U94>(&opt),
			48 => process::<U48, U96>(&opt),
			49 => process::<U49, U98>(&opt),
			50 => process::<U50, U100>(&opt),
			51 => process::<U51, U102>(&opt),
			52 => process::<U52, U104>(&opt),
			53 => process::<U53, U106>(&opt),
			54 => process::<U54, U108>(&opt),
			55 => process::<U55, U110>(&opt),
			56 => process::<U56, U112>(&opt),
			57 => process::<U57, U114>(&opt),
			58 => process::<U58, U116>(&opt),
			59 => process::<U59, U118>(&opt),
			60 => process::<U60, U120>(&opt),
			61 => process::<U61, U122>(&opt),
			62 => process::<U62, U124>(&opt),
			63 => process::<U63, U126>(&opt),
			64 => process::<U64, U128>(&opt),
			_ => {
				eprintln!("Only M between 1 and 64 inclusive are allowed");
				return;
			}
		}
	};

	let mut fg = Figure::new();

	fg.axes2d()
		.set_title(
			&format!("1-NN Recall Graph (M = {}, size = {})", opt.m, opt.size),
			&[],
		)
		.set_x_label("Recall Rate", &[])
		.set_y_label("Lookups per second", &[])
		.lines(&recalls, &times, &[LineWidth(2.0), Color("blue")])
		.set_y_ticks(Some((Auto, 2)), &[], &[])
		.set_grid_options(true, &[LineStyle(DotDotDash), Color("black")])
		.set_minor_grid_options(&[LineStyle(SmallDot), Color("red")])
		.set_x_grid(true)
		.set_y_grid(true)
		.set_y_minor_grid(true);

	fg.show();
}