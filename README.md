# picas_static_phase

This tool reproduces the priority and affinity settings of the PiCAS framework [1] using multiple DAGs as input. The results are output in the form of a [ros2_thread_configurator](https://github.com/sykwer/ros2_thread_configurator) configuration file.

## Input DAG file format
The input DAG must be in yaml format, as in the [autoware_dags](https://github.com/atsushi421/picas_static_phase/tree/main/autoware_dags) directory. [autoware_dags/visualize_dag.py](https://github.com/atsushi421/picas_static_phase/tree/main/autoware_dags/visualize_dag.py) is a visualization script to check the correctness of DAG yaml files.

## Usage

```
$ cd picas_static_phase
$ cargo run -- -d <DAG_DIR_PATH> -c <NUMBER_OF_CORES> -o <OUTPUT_DIR_PATH>
```

The details of each option are as follows:

```
-d, --dag_dir <DAG_DIR_PATH>             Path to DAGSet directory [default: ../autoware_dags]
-c, --number_of_cores <NUMBER_OF_CORES>  Number of processing cores [default: 16]
-o, --output_path <OUTPUT_DIR_PATH>      Path to output directory [default: ../]
```

## Reference
1. Choi, H., Xiang, Y., & Kim, H. (2021). PiCAS: New design of priority-driven chain-aware scheduling for ROS2. In Proceedings of the 2021 IEEE 27th Real-Time and Embedded Technology and Applications Symposium (RTAS) (pp. 251-263). IEEE.
