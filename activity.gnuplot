#set term svg
#set output 'activity.svg'
set style data histogram
set style histogram cluster gap 1
set xrange [0:24]
plot 'activity.csv' using 2
