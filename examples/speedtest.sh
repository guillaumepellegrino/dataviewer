#!/bin/bash


speedtest_load()
{
    cat <<EOF
[Load.dataview]
type = "XY"
title = "Speedtest"
x_title = "Time"
x_unit = "seconds"
y_title = "Throughput"
y_unit = "Mbps"
description = """
TCP Speedtest download and upload throughput,
measured with iperf3
"""

[Load.chart.1]
title = "Download"
description = """TCP Speedtest download throughput
measured with iperf3
"""

[Load.chart.2]
title = "Upload"
description = """TCP Speedtest upload throughput
measured with iperf3
"""
EOF
    printf "\0"
}

main()
{
    speedtest_load

    time=0
    while true; do
        download=$(((RANDOM % 40) + 800))
        upload=$(((RANDOM % 10) + 80))
        echo "[Update]"
        echo "1.data=[$time,$download]"
        echo "2.data=[$time,$upload]"
        printf "\0"
        sleep 1
        time=$((time+1))
    done
}

main | nc -U /tmp/dataviewer.ipc
