# Route

A sequence of _legs_, where each leg connects on fix point with
another. Each route *must* have an origin and destination.

## Conditional Leg Values

The following table lists leg values that are not displayed if the
required values are missing:

| Value           | Required                        |
|-----------------|---------------------------------|
| WCA             | Speed, Wind                     |
| GS              | Speed, Wind                     |
| Heading         | Speed, Wind                     |
| ETE             | Speed, Wind                     |
| Fuel            | Speed, Wind, Cruise Performance |
| Magnetic Course | Valid Date & Time               |

## Changing Speed, Level & Wind Mid-Route

Performance parameters apply from where they appear until changed:

    13509KT N0107 EDDH D DCT 18009KT DCT W EDHL

- Leg EDDH → D uses wind from 135°
- Legs D → W and W to EDHL use wind from 180°

You can also change speed or altitude mid-route:

    N0107 A025 EDDH P2 A045 N0120 EDHF

- Leg EDDH → P2 at 107 kt and 2500 ft
- Leg P2 → EDHF at 120 kt and 4500 ft
