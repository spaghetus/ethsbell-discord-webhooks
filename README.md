# ethsbell discord webhooks

This program outputs a crontab which will output discord webhooks for ethsbell.

To set it up:

* Install curl
* Put this program's binary somewhere memorable
* Create a file with a list of all discord webhooks to use
* Add a line to the root crontab like this: (all arguments are optional)
```crontab
0 0 */5 * * bash -c '/bin/ethsbell-discord-webhooks -n 7 -i /usr/share/bell-webhook-urls -t 0 -u https://ethsbell.app/api/v1/schedule > /etc/cron.d/ethsbell.crontab'
```