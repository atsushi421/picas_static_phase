import os
import networkx as nx
import matplotlib.pyplot as plt
import pydot

def convert_to_nx_graph(dot_content):
    pydot_graph = pydot.graph_from_dot_data(dot_content)[0]
    return nx.nx_pydot.from_pydot(pydot_graph)

def extract_correct_name(label):
    try:
        return label.split('name: ')[1].split(',')[0].replace('"', '')
    except IndexError:
        return ""

def vertical_layout(G):
    in_degrees = dict(G.in_degree())
    sorted_nodes = sorted(G.nodes(), key=lambda n: (in_degrees[n], n))

    pos = {}
    num_nodes = len(sorted_nodes)
    for idx, node in enumerate(sorted_nodes):
        y_coord = num_nodes - idx
        pos[node] = (1, y_coord)  # all nodes have the same x-coordinate

    return pos

# Fetch all the dot files in the current directory
dot_files = [file for file in os.listdir() if file.endswith('.dot')]

for dot_file in dot_files:
    with open(dot_file, "r") as file:
        dot_content = file.read()

    G_nx = convert_to_nx_graph(dot_content)
    correct_node_labels = {node: extract_correct_name(data['label']) for node, data in G_nx.nodes(data=True)}
    vertical_pos = vertical_layout(G_nx)

    plt.figure(figsize=(8, 14))
    nx.draw(G_nx, vertical_pos, labels=correct_node_labels, with_labels=True, node_size=3000, node_color="skyblue", font_size=10)
    plt.title(f"Vertical DAG Visualization of {dot_file} with Corrected 'name' as Labels")

    # Save the figure as a PDF
    output_pdf_name = os.path.splitext(dot_file)[0] + ".pdf"
    plt.savefig(output_pdf_name, format='pdf', bbox_inches='tight')
    plt.close()
