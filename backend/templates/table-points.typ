        #table(
            columns: (auto,auto,auto,auto,auto),
            inset: 3pt,
            stroke: 0.2pt,
            align: (x, y) => (
                if x == 0 { center }
                else { right }
            ),
            [*name*],[*km*],[*time*],[*D+*],[*%*],
            /* #line-template [{name}],[{distance}],[{time}],[{d+}],[{slope}], */
        )
