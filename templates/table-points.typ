        #table(
            columns: (auto,auto,auto,auto),
            inset: 3pt,
            stroke: 0.2pt,
            align: (x, y) => (
                if x == 1 { right }
                else { center }
            ),
            [*name*],[*distance*],[*time*],[*D+*],
            /* #line-template [{name}],[{distance}],[{time}],[{d+}], */
        )
