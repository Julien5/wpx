#set text(
    font: "Liberation Mono",
    size: 9pt
)

#set page(margin: (
  top: 1cm,
  bottom: 1cm,
  x: 1cm,
))

#set table.hline(stroke: .6pt)

#table(
    columns: 2,
    inset: 0pt,
    stroke: 1pt,
    align: (center + horizon,center+horizon),
    table.cell(colspan:2,image("/tmp/profile.svg", width: 100%)),
    table.cell(colspan:1,inset:10pt,image("/tmp/map.svg", height: 100pt, width:200pt)),
    table.cell(colspan:1,inset:10pt,[
        #table(
            columns: (auto,auto,auto,auto,auto),
            inset: 3pt,
            stroke: 0.2pt,
            align: horizon,
            [*name*],[*distance*],[*time*],[*D+*],[*slope*],
                    
            [K1],[10],[01:10],[100],[10],
            [A1],[15],[02:50],[150],[12]
        )
    ]),
    table.hline()
)

