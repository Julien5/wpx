set macros
set datafile separator comma
if (!exists("xrange")) xrange="[*:*]"
if (!exists("filename")) filename="processing.csv"
if (exists("pngfile")) set terminal pngcairo size 1000,800
if (exists("pngfile")) set output pngfile

set grid xtics
set grid ytics

set multiplot layout 3,1

set xlabel "Time (hours)"
set ylabel "Smooth speed (km/h)"
set xrange @xrange
plot filename using 1:5 with lines lc rgb "blue" title "smooth speed"

set xlabel "Time (hours)"
set ylabel "Slope (%)"
set xrange @xrange
plot filename using 1:6 with lines lc rgb "blue" title "slope"

set xlabel "Time (hours)"
set ylabel "Power (W)"
set xrange @xrange
set yrange [0:400]
plot filename using 1:11 with lines lc rgb "blue" title "measured", \
     filename using 1:12 with lines lc rgb "red" title "estimate"

unset multiplot
