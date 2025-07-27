        #table(
            columns: (10mm,auto,15mm,auto,10mm,auto,auto),
            inset: (x: 2mm,y:1mm),
            stroke: 0.2pt,
            align: (x, y) => (
                if x == 0 { right }
                else if x == 6 { left }
                else { right }
            ),
            [*KM*],[*TIME*],[*HM*],[*DIST*],[*D+*],[*SLOPE*],[*DESCRIPTION*],
            /* #line-template [{distance}],[{time}],[{elevation}],[{dist}],[{d+}],[{slope}],[{desc}], */
        )
