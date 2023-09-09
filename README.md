
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

## Watch SpeedTest results in real-time

https://github.com/guillaumepellegrino/dataviewer/blob/master/examples/speedtest.sh
