set macros
set datafile separator comma
if (!exists("xrange")) xrange="[*:*]"
if (!exists("filename")) filename="processing.csv"
if (exists("pngfile")) set terminal pngcairo size 1000,800
if (exists("pngfile")) set output pngfile

set multiplot layout 3,1

set xlabel "Time (hours)"
set ylabel "Distance (km)"
set xrange @xrange
plot filename using 1:2 with lines lw 3 lc rgb "blue" title "input"

set xlabel "Time (hours)"
set ylabel "Elevation (m)"
set xrange @xrange
plot filename using 1:3 with lines lw 3 lc rgb "blue" title "input"

set xlabel "Time (hours)"
set ylabel "Slope (%)"
set xrange @xrange
plot filename using 1:6 with lines lc rgb "red" title "output"

unset multiplot
