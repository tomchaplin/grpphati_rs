from grpphati_rs.grpphati_rs import RustIteratorSparsifier
from grpphati.sparsifiers import Sparsifier


class RustGeneratorSparsifier(Sparsifier):
    def __init__(self, return_dimension=False):
        self.return_dimension = return_dimension

    def __call__(self, cols):
        self.internal_sparsifier = RustIteratorSparsifier(list(cols))
        return self

    def __iter__(self):
        return self

    def __next__(self):
        next_col = self.internal_sparsifier.get_next()
        if next_col is None:
            raise StopIteration
        if self.return_dimension:
            return next_col
        else:
            return next_col[1]
