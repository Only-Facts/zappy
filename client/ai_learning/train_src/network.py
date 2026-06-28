import numpy as np

def compute_genome_size(layer_dims):
    size = 0
    for i in range(len(layer_dims) - 1):
        size += layer_dims[i] * layer_dims[i+1]
        size += layer_dims[i+1]
    return size

class MLP:
    def __init__(self, genome, layer_dims):
        self.genome = genome
        self.layer_dims = layer_dims

        self.weights = []
        self.biases = []

        self._decode_genome()

    def _decode_genome(self):
        idx = 0

        for in_dim, out_dim in zip(self.layer_dims[:-1], self.layer_dims[1:]):
            w = self.genome[idx:idx + in_dim * out_dim]
            w = w.reshape(out_dim, in_dim)
            idx += in_dim * out_dim

            b = self.genome[idx:idx + out_dim]
            idx += out_dim

            self.weights.append(w)
            self.biases.append(b)

    @classmethod
    def random(cls, layer_dims, std=0.1):
        genome = np.random.randn(compute_genome_size(layer_dims)) * std
        return cls(genome, layer_dims)

    def forward(self, x):
        x = np.asarray(x, dtype=np.float32)

        for W, b in zip(self.weights[:-1], self.biases[:-1]):
            x = np.maximum(0, W @ x + b)

        return self.weights[-1] @ x + self.biases[-1]
    
    def fork(self, mutation_std=0.01):
        new_genome = (self.genome + np.random.randn(len(self.genome)) * mutation_std)
        return MLP(new_genome, self.layer_dims)
