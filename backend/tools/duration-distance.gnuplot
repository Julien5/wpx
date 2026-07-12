set datafile separator "|"
set xdata time
set timefmt "%H:%M:%S"
set format x "%Hh"
set xtics 3600
set ytics 50
set format y "%.0f km"
set grid
set terminal png size 800,800
set output "/tmp/duration-distance/duration-distance.png"
set key autotitle columnheader
plot for [i=2:*] "/tmp/duration-distance/duration-distance.csv" using 1:(column(i)/1000.0) with lines
