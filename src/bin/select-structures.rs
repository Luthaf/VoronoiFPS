use clap::{Arg, App};

use ndarray::{Array1, Array2};
use ndarray_npy::{read_npy, write_npy};

use voronoi_fps::VoronoiDecomposer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new("select-structures")
        .author("Guillaume Fraux <guillaume.fraux@epfl.ch>")
        .about(
"Select training points from a dataset using a Voronoï realization of FPS.

This tool automatically select adds all environments from a structure when any
environment in this structure is selected."
        )
        .arg(Arg::with_name("structures")
            .long("structures")
            .value_name("structures.npy")
            .help("array of structure indexes")
            .takes_value(true)
            .required(true))
        .arg(Arg::with_name("n_structures")
            .short("n")
            .help("how many structures to select")
            .takes_value(true)
            .required(true))
        .arg(Arg::with_name("output")
            .short("o")
            .long("output")
            .value_name("output.npy")
            .help("where to output selected structures indexes")
            .takes_value(true)
            .required(true))
        .arg(Arg::with_name("output-radius")
            .long("radius")
            .value_name("radius.npy")
            .help("where to output Voronoi radii of selected points")
            .takes_value(true)
            .required(true))
        .arg(Arg::with_name("points")
            .long("points")
            .value_name("points.npy")
            .help("Sets the input file to use")
            .takes_value(true)
            .required(true))
        .get_matches();

    let points: Array2<f64> = read_npy(matches.value_of("points").unwrap())?;
    let structures: Array1<u32> = read_npy(matches.value_of("structures").unwrap())?;
    let n_select: usize = matches.value_of("n_structures").unwrap().parse()?;

    let initial = 0;
    let mut voronoi = VoronoiDecomposer::new(points.view(), initial);
    let mut radius_when_selected = Vec::new();
    radius_when_selected.push(*voronoi.cells().last().unwrap().radius2);

    for point in structures.iter()
        .enumerate()
        .filter_map(|(i, &s)| {
            if s == structures[initial] && i != initial {
                Some(i)
            } else {
                None
            }
        }) {
                voronoi.add_point(point);
                radius_when_selected.push(*voronoi.cells().last().unwrap().radius2);
            }

    let mut selected_structures = vec![structures[initial]];

    for _ in 1..n_select {
        let selected_point = {
            let cells = voronoi.cells();
            let (max_radius_cell, radius) = find_max(cells.radius2.iter());
            radius_when_selected.push(radius);

            cells.farthest[max_radius_cell]
        };

        voronoi.add_point(selected_point);

        let selected = structures[selected_point];
        selected_structures.push(selected);
        for point in structures.iter()
            .enumerate()
            .filter_map(|(i, &s)| {
                if s == selected && i != selected_point {
                    Some(i)
                } else {
                    None
                }
            }) {
                voronoi.add_point(point);
                radius_when_selected.push(*voronoi.cells().last().unwrap().radius2);
            }
    }

    let selected_structures = Array1::from(selected_structures);
    write_npy(matches.value_of("output").unwrap(), &selected_structures)?;

    let radius_when_selected = Array1::from(radius_when_selected);
    write_npy(matches.value_of("output-radius").unwrap(), &radius_when_selected)?;

    return Ok(());
}


/// Get both the maximal value in `values` and the position of this maximal
/// value
fn find_max<'a, I: Iterator<Item=&'a f64>>(values: I) -> (usize, f64) {
    values
        .enumerate()
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).expect("got NaN value"))
        .map(|(index, value)| (index, *value))
        .expect("got an empty slice")
}