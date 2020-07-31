Uniden UBC 125XLT Communications Protocol
=========================================
The UBC 125XLT presents itself as a USB device with a single configuration and
two different interfaces. The first interface is a Communications class interface.
Possibly it can be used to control the scanner directly. The second interface
(CDC Data class) can be used to program the scanner using bulk transfers.

The CDC Data interface has two endpoints: one for writing data to the scanner
(0x02) and one for reading data from the scanner (0x81).

Generic message format
----------------------
Each message consists of a three letter code, followed by zero or more fields,
and end with a CR ('\r'). Fields are separated by commas (',') in between fields.
If there are zero fields, no comma is necessary and messages should not end with
a comma before the CR. For example, to download channel 5 from the scanner, the
appropriate message would exists of the three letter combination "CIN", the field
"5" and end with a CR. It would look like this: "CIN,5\r".
Messages always occur in pairs. A requests is sent to the scanner, which results
in a reply from the scanner. Both requests and replies share the same three letter
comibination.
Please note that messages (especially replies) can be split over multiple bulk
transfers.

Known messages
--------------

### Model information (MDL)
These messages use the three letter code "MDL". Requests take no fields and the
response has a single field containing a string describing the device model (i.e.
"UBC125XLT").

### Firmware version (VER)
These messages use the three letter code "VER". Requests take no fields and the
response has a single field containing a string with the version number of the
firmware.

### Enter programming mode (PRG)
These messages use the three letter code "PRG". Requests take no fields and the
response has a single "OK" field if the operation was successful.

In order to program the scanner, it has to be in programming mode (shown as
"Remote Mode" on the display of the scanner). While the scanner is in this mode,
it cannot be operated manually.

### Leave programming mode (EPG)
These messages use the three letter code "EPG". Requests take no fields and the
response has a single "OK" field if the operation was successful.

In order to program the scanner, it has to be in programming mode (shown as
"Remote Mode" on the display of the scanner). While the scanner is in this mode,
it cannot be operated manually.

### Download channel information (CIN)
These messages use the three letter code "CIN". Requests take a single field which
contains the channel number. The channel number is a number between 1 and 500.
E.g. Bank 3 Channel 2 would be channel number 3\*50 + 2 = 152.

Responses contain 8 fields:
* The channel id
* The tag
* The frequency in 10^2 Hz (e.g. a value of 1 would mean a frequency of 100 Hz)
* The modulation used (AM or FM)
* Information on the CTCSS / DCS signal that is used
* The delay in seconds
* A 1 if the channel is locked, a 0 otherwise.
* A 1 if the channel has priority, a 0 otherwise.

### Upload channel information (CIN)
These messages use the three letter code "CIN". Requests take eight fields, as
described in the download section.

The response contains a single "OK" field if the upload was successful.
