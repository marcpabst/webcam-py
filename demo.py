# Copyright (c) 2024 Marc Pabst
# 
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.


import time
import os

# add the gstreamer bin folder to the path (C:\gstreamer\1.0\msvc_x86_64\bin)
os.add_dll_directory(r"C:\gstreamer\1.0\msvc_x86_64\bin")

import webcam_py

# create camera caps object
camera_caps = webcam_py.CameraCaps(
    width=1280,
    height=720,
    framerate_numerator=30,
    framerate_denominator=1,
    format="NV12",
)

# start recordingß
recorder = webcam_py.start_recording(camera_caps, "test23.mkv")

# print 10 000 messages
for i in range(10):
    print("Hello World Number ", i)
    # sleep for 1 second
    time.sleep(1)


# stop recording
webcam_py.stop_recording(recorder)
