#!/bin/bash


speedtest_load()
{
    cat <<EOF
[dataview]
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

[chart.1]
title = "Download"
description = """TCP Speedtest download throughput
measured with iperf3
"""

[chart.2]
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
        echo "[data]"
        echo "1=[$time,$download]"
        echo "2=[$time,$upload]"
        printf "\0"
        sleep 1
        time=$((time+1))
    done
}

main | nc -U /tmp/dataviewer.ipc
