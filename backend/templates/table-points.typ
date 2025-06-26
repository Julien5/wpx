        #table(
            columns: (auto,auto,auto,auto,auto,auto,auto,auto),
            inset: 3pt,
            stroke: 0.2pt,
            align: (x, y) => (
                if x == 0 { center }
                else { right }
            ),
            [*NAME*],[*KM*],[*TIME*],[*HM*],[*DIST*],[*D+*],[*SLOPE*],[*DESCRIPTION*],
            /* #line-template [{name}],[{distance}],[{time}],[{elevation}],[{dist}],[{d+}],[{slope}],[{desc}], */
        )
