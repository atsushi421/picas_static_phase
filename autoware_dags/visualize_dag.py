import networkx as nx
import matplotlib.pyplot as plt
import yaml
import os
import glob

def top_down_layout_more_spacing(G):
    # Start with the nodes without predecessors
    root_nodes = [node for node in G.nodes() if not list(G.predecessors(node))]

    y_spacing = -10.5  # Further adjusted the spacing
    x_spacing = 1

    pos = {}
    x_offset = 0

    visited = set()

    def place_node(node, x, y):
        pos[node] = (x, y)
        visited.add(node)
        children = [n for n in G.successors(node) if n not in visited]
        y_next = y + y_spacing
        x_next = x - (x_spacing * len(children)) / 2 + x_spacing / 2
        for child in children:
            place_node(child, x_next, y_next)
            x_next += x_spacing

    for root_node in root_nodes:
        place_node(root_node, x_offset, 0)
        x_offset += x_spacing * max(1, len(list(G.successors(root_node))))

    return pos

def visualize_graph_top_down_more_spacing(yaml_path):
    # Load the YAML file content
    with open(yaml_path, 'r') as file:
        perception_data = yaml.safe_load(file)

    # Create a directed graph
    G = nx.DiGraph()

    # Add nodes and store wcet as attribute
    for node in perception_data['nodes']:
        G.add_node(node['id'], name=node['name'], wcet=node['wcet'])

    # Add edges
    for link in perception_data['links']:
        G.add_edge(link['source'], link['target'])

    # Using top-down layout with more spacing
    pos = top_down_layout_more_spacing(G)

    # Plot the graph
    plt.figure(figsize=(15, 10))
    labels = {node: f"{G.nodes[node]['name']}\n(WCET: {G.nodes[node]['wcet']}ms)" for node in G.nodes()}
    nx.draw(G, pos, labels=labels, node_size=1500, node_color='skyblue', font_size=6, font_weight='bold', width=1.5, edge_color='gray')
    plt.title(f'Top-Down Graph Structure with WCET of {os.path.basename(yaml_path)}')

    # Determine the output PDF path (in the same directory as the input YAML, but with a .pdf extension)
    output_pdf_path = os.path.splitext(yaml_path)[0] + ".pdf"

    # Save the graph as a PDF image
    plt.savefig(output_pdf_path, format='pdf')
    plt.close()

if __name__ == "__main__":
    current_directory = os.path.dirname(os.path.realpath(__file__))
    yaml_files = glob.glob(os.path.join(current_directory, "*.yaml"))

    for yaml_file in yaml_files:
        visualize_graph_top_down_more_spacing(yaml_file)
