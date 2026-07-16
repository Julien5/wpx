set macros
set datafile separator comma
if (!exists("xrange")) xrange="[*:*]"
if (!exists("filename")) filename="processing.csv"
if (exists("pngfile")) set terminal pngcairo size 1000,800
if (exists("pngfile")) set output pngfile

set multiplot layout 2,1

set xlabel "Time (hours)"
set ylabel "Distance (km)"
set xrange @xrange
plot filename using 1:7 with lines lc rgb "black" title "raw", \
     filename using 1:9 with points pt 7 ps 0.5 lc rgb "red" title "oversampled"

set xlabel "Time (hours)"
set ylabel "Elevation (m)"
set xrange @xrange
plot filename using 1:8 with lines lc rgb "black" title "raw", \
     filename using 1:10 with points pt 7 ps 0.5 lc rgb "red" title "oversampled"

unset multiplot
