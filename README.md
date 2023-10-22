# wol-server

Use it as follow:
Make a request to `localhost:4441/00_0a_0b_0c_0d_0e`

The server will send a wake-on-lan magic packet to `00:0a:0b:0c:0d:0e`.
The server will send up to 1000 magic packets if some fail. Each with an delay of 5ms.
If every sent packet is able to be sent, it sends 100 packets to make sure the device is really noticing and waking.

If you have further questions, please look into issues or create one.

License: GPL-3.0

Author: Paul Barbenheim

