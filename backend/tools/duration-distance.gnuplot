set datafile separator "|"
set xlabel "Time (hours)"
set xtics 1
set ytics 50
set format y "%.0f km"
set grid
set terminal png size 1800,800
set output "/tmp/duration-distance/duration-distance.png"
set key autotitle columnheader
plot for [i=2:*] "/tmp/duration-distance/duration-distance.csv" using 1:(column(i)/1000.0) with lines
