        #table(
            columns: (10mm,15mm,auto,auto),
            inset: (x: 2mm,y:1mm),
            stroke: 0.2pt,
            align: (x, y) => (
                if x <= 1 { center }
                else { left }
            ),
            [*KM*],[*TIME*],[*NAME*],[*DESCRIPTION*],
            /* #line-template [{distance}],[{time}],[{name}],[{description}], */
        )
