
# Watch and view your data in real-time.
This application allow you to watch your data in real-time (using a simple and documented protocol) or view them from files.

This crate make it easy to display the data from third-party applications.

GTK4 must be installed to build this application:
```
sudo apt install libgtk-4-dev
```


(under development)


# DataView format
DataView format is a user-friendly format based on TOML.

- Usage examples are available here:
https://github.com/guillaumepellegrino/dataviewer/tree/master/examples

- Format is defined with SERDE here:
https://github.com/guillaumepellegrino/dataviewer/blob/master/src/dataview.rs

# Examples
## View SpeedTest results
![alt text](https://github.com/guillaumepellegrino/dataviewer/blob/master/images/DataViewerSpeedTest.png)
https://github.com/guillaumepellegrino/dataviewer/blob/master/examples/speedtest.dv.toml

## Streaming SpeedTest in real-time

![alt text](https://github.com/guillaumepellegrino/dataviewer/blob/master/images/WatchSpeedTest.gif)
https://github.com/guillaumepellegrino/dataviewer/blob/master/examples/speedtest.sh

Data can be streamed in real-time through an ipc socket defined in /tmp/dataviewer.ipc.
The ipc is using the same format than files. The only difference is each message/update must be termined by a NULL character.
So, you may very well cat your dataview file in the ipc terminated by a '\0' followed up by updates each terminated by a '\0'.

## View Top memory allocations over time with memtrace
![alt text](https://github.com/guillaumepellegrino/dataviewer/blob/master/images/DataViewerMemtrace.png)

https://github.com/guillaumepellegrino/memtrace/blob/v1.1.0/src/agent.c#L359
