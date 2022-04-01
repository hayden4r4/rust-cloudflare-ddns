# A DDNS script for cloudlfare in Rust #  
  
This is a simple script that updates your DNS domain's IP when the host IP changes.  The only setup required is to create a new entry 'A' (for ipv4) or 'AAAA' (for ipv6) entry in cloudlfare with your desired name (ex. name=ddns for ddns.mywebsite.com) and 0.0.0.0 for content with proxy disabled.  Then create a .env file with your settings in the root of the project folder. Finally, schedule this script to run at desired interval (I recommend between 5-30 mins).  
  
### The .env file should be set up as follows: ###  
TOKEN: a string of your cloudlfare api key configured with the 'edit zone dns' template and with your domain name selected as the zone resource.  This string should be appended by 'Bearer '.  This is used to authenticate with the cloudflare API. ex: 'Bearer dslfhasd34fsfsdlfhlska'  
TYPE: a string, either 'A' or 'AAAA' for ipv4 or ipv6 respectively.  
RECORD: a string of the full url of your domain.  ex: 'ddns.mywebsite.com'  
ZONEID: a string of your Zone ID which can be found on the overview page of your domain on cloudflare's website.  
ID: a string of your identifier.  This is the most difficult to get however it is static so only needs to be obtained once.  I recommend making a quick curl call with the method [here](https://api.cloudflare.com/#dns-records-for-a-zone-list-dns-records).  Here is the call for reference, fill in the <> with your info, look for the 'id' field for your RECORD:  
    curl -X GET "https://api.cloudflare.com/client/v4/zones/<ZONE_ID>/dns_records?type=<TYPE>&name=<RECORD>&content=0.0.0.0" \  
    -H "Authorization: <TOKEN>" \  
    -H "Content-Type: application/json"  
  
Having the script run automatically will depend on your platform, but assuming your using linux you can simply open a terminal:  
1. run 'crontab -e' and select your editor  
2. add '*/5  * * * * path/to/executable' without the quotes, this will run the script every 5 mins.  Feel free to replace 5 with the desired interval of minutes.  
  
And that's it.  Be sure to check your cloudlfare dns dashboard to make sure that it has updated properly.