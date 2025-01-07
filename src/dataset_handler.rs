use nd::{s, Array2, ArrayD};
use npy::ReadNpyExt;
use polars::prelude::*;
use std::fs::File;
use std::path::{Path, PathBuf};

pub fn load_rssi_dataset(
    path_to_csv: &str,
    test_proportion: f64,
) -> (ArrayD<f64>, ArrayD<f64>, ArrayD<f64>, ArrayD<f64>) {
    assert!(
        0. < test_proportion && test_proportion < 1.,
        "The test proportion should be in the (0, 1) range."
    );
    let df = CsvReadOptions::default()
        .with_has_header(true)
        .try_into_reader_with_file_path(Some(path_to_csv.into()))
        .unwrap()
        .finish()
        .unwrap()
        .to_ndarray::<Float64Type>(IndexOrder::C)
        .unwrap();
    let df_nrows = df.nrows();
    let mut y_matrix = Array2::zeros((df_nrows, 2));
    let mut x_matrix = Array2::zeros((df_nrows, 13));
    for i in 0..df_nrows {
        for j in 1..=2 {
            y_matrix[[i, j - 1]] = df[[i, j]];
        }
        for j in 3..=15 {
            x_matrix[[i, j - 3]] = df[[i, j]];
        }
    }
    let num_test = (df_nrows as f64 * test_proportion).round() as usize;
    let num_train = df_nrows - num_test;
    let train_slice = s![num_train..df_nrows, ..];
    let test_slice = s![num_train..df_nrows, ..];
    let x_train = x_matrix.slice(train_slice).into_owned().into_dyn();
    let y_train = y_matrix.slice(train_slice).into_owned().into_dyn();
    let x_test = x_matrix.slice(test_slice).into_owned().into_dyn();
    let y_test = y_matrix.slice(test_slice).into_owned().into_dyn();
    (x_train, y_train, x_test, y_test)
}

fn read_mnist_npy(path_to_npy: PathBuf) -> ArrayD<f64> {
    let reader = File::open(path_to_npy).expect("Failure when reading npy file.");
    let array = ArrayD::<f64>::read_npy(reader).expect("Failure when parsing npy file.");
    array
}

pub fn load_mnist_dataset(
    path_to_folder: &str,
) -> (ArrayD<f64>, ArrayD<f64>, ArrayD<f64>, ArrayD<f64>) {
    let files = ["x_train.npy", "y_train.npy", "x_test.npy", "y_test.npy"];
    let path_to_folder = Path::new(path_to_folder);
    let mut arrays = [None, None, None, None];
    for (i, &file) in files.iter().enumerate() {
        let file = Path::new(file);
        let filepath = path_to_folder.join(file);
        let array = read_mnist_npy(filepath);
        arrays[i] = Some(array);
    }
    let (x_train, y_train, x_test, y_test) = (
        arrays[0].take().unwrap(),
        arrays[1].take().unwrap(),
        arrays[2].take().unwrap(),
        arrays[3].take().unwrap(),
    );
    (x_train, y_train, x_test, y_test)
}
