Standard = { $OptionCategory ->
    *[some] Standard
    [none] Without a standard
}
    .hover = Fatty acid added as a standard.

Composition = { $PluralCategory ->
    *[one] Composition
    [other] Compositions
}
    .hover = [stereospecific, positional, non-stereospecific], [mass, ECN, species, type, unsaturation] composition.

Correlation = { $PluralCategory ->
    *[one] Correlation
    [other] Correlations
}
    .hover = Pearson or spearman rank correlation

Index = { $PluralCategory ->
    *[one] Index
    [other] Indices
}
    .hover = Desctiption for `Index`

StereospecificNumber = { $number ->
    [1] Stereospecific number 1
    [2] Stereospecific number 2
    [3] Stereospecific number 3
    *[one] Stereospecific number
    [123] Stereospecific numbers 1, 2, 3
    [1223] Stereospecific numbers 1, 2 or 2, 3
    [13] Stereospecific numbers 1, 3
    [many] Stereospecific numbers
}
    .abbreviation = { $number ->
        [1] SN-1
        [2] SN-2
        [3] SN-3
        [123] SN-1,2,3
        [1223] SN-1,2(2,3)
        [13] SN-1,3
        *[other] SN
    }

Property = { $PluralCategory ->
    *[one] Property
    [other] Properties
}
    .hover = Properties.
