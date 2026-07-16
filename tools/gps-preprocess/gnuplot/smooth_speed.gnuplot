set macros
set datafile separator comma
if (!exists("xrange")) xrange="[*:*]"
if (!exists("filename")) filename="processing.csv"
if (exists("pngfile")) set terminal pngcairo size 1000,800
if (exists("pngfile")) set output pngfile

set xlabel "Time (hours)"
set ylabel "Speed (km/h)"
set xrange @xrange
plot filename using 1:4 with lines lw 3 lc rgb "blue" title "input", \
     filename using 1:5 with lines lc rgb "red" title "smooth speed"
