1.
While using app: com.amazon.mShop.android.shopping
Uri: https://www.amazon.ca/service-worker.js
http3 request has different header:

Header: cookie
Value in http3:
amzn-app-ctxt=1.8%20%7B%22an%22%3A%22Amazon.com%22%2C%22av%22%3A%2228.21.0.100%22%2C%22xv%22%3A%221.15.0%22%2C%22os%22%3A%22Android%22%2C%22ov%22%3A%2213%22%2C%22cp%22%3A788760%2C%22uiv%22%3A4%2C%22ast%22%3A3%2C%22nal%22%3A%221%22%2C%22di%22%3A%7B%22pr%22%3A%22oriole%22%2C%22md%22%3A%22Pixel%206%22%2C%22v%22%3A%22oriole%22%2C%22mf%22%3A%22Google%22%2C%22dsn%22%3A%2229f2d29308c149b88495382de8feed61%22%2C%22dti%22%3A%22A1MPSLFC7L5AFK%22%2C%22ca%22%3A%22%22%2C%22ct%22%3A%22WIFI%22%7D%2C%22dm%22%3A%7B%22w%22%3A1080%2C%22h%22%3A2209%2C%22ld%22%3A2.625%2C%22dx%22%3A409.4320068359375%2C%22dy%22%3A411.8909912109375%2C%22pt%22%3A0%2C%22pb%22%3A78%7D%2C%22is%22%3A%22com.android.vending%22%2C%22msd%22%3A%22.amazon.ca%22%7D

with url decode:
{
   "an":"Amazon.com",
   "av":"28.21.0.100",
   "xv":"1.15.0",
   "os":"Android",
   "ov":"13",
   "cp":788760,
   "uiv":4,
   "ast":3,
   "nal":"1",
   "di":{
      "pr":"oriole",
      "md":"Pixel 6",
      "v":"oriole",
      "mf":"Google",
      "dsn":"29f2d29308c149b88495382de8feed61",
      "dti":"A1MPSLFC7L5AFK",
      "ca":"",
      "ct":"WIFI"
   },
   "dm":{
      "w":1080,
      "h":2209,
      "ld":2.625,
      "dx":409.4320068359375,
      "dy":411.8909912109375,
      "pt":0,
      "pb":78
   },
   "is":"com.android.vending",
   "msd":".amazon.ca"
}

there are some device information in this body



2.
While using app: com.amazon.mShop.android.shopping
Uri: https://m.media-amazon.com/images/I/61K6ulicWAL._AC_SR438,658_QL65_.jpg

http3 request has different header:

Header: priority
Value in http3:
i

Value in http2:
u=1, i

also, many requests belong to google and amazon use different version of chrome to render,
for example, in user-agent field of a request header:
Mozilla/5.0 (Linux; Android 13; Pixel 6 Build/TQ3A.230901.001; wv) AppleWebKit/537.36 (KHTML, like Gecko) Version/4.0 Chrome/129.0.6668.100 Mobile Safari/537.36

chrome version 129 for http3, and 130 for http2



3.
While using app: com.alibaba.intl.android.apps.poseidon
Uri: https://play.google.com/log?format=json&hasfast=true&authuser=0

http3 request has distinct header:

Http3 specific Header: cookie

Captured value of this header:
__Secure-3PSIDCC=AKEyXzU5i3w01bZR2CU7gksoQipltCFZovWqYVN-08t2AzBfim3Ozp84-oXcT-0k5iDLUyjC


Http3 specific Header: authorization

Captured value of this header:
SAPISIDHASH 40f4f12cc1783e115f976006daf7fabf8dbd1e54

http3 request has specific body:
[[1,null,null,null,null,null,null,null,null,null,[null,null,null,null,"en-US",null,"boq_identity-oauth-http_20241029.02_p0",null,[[["Android WebView","129"],["Not=A?Brand","8"],["Chromium","129"]],1,"Android","13.0.0","","Pixel 6","129.0.6668.100"],[4,0,0,0,0]]],1022,[["1730759135077",null,null,null,null,null,null,null,null,null,null,null,null,null,18000,null,null,null,null,null,2,null,null,"[[[1730759133024000,0,0],2],null,[[1730759133024000,0,0],1],[191077,2,3,null,null,null,null,null,null,null,null,null,null,[[188412,{\"672\":[null,null,[[\"m4yhhGnL8i50qN6FqfgVog\"]]]}],[191077,{\"672\":[null,null,[[\"m4yhhGnL8i50qN6FqfgVog\"]]]}]],[{\"126\":[[\"1730759135077\"]]}]],null,null,null,null,[[[1730759133024000,0,0],1]]]"]],"1730759135078",null,null,null,null,null,null,null,null,null,null,null,null,null,[[null,[null,null,null,null,null,null,null,null,null,null,null,null,128566913]],9]]

This google play link only appeared once, and with protocol HTTP3



4.
While using app: com.pinterest
Uri: https://v1.pinimg.com/videos/mc/h265/b5/d2/53/b5d253a36fec1f8a2cfded9565be8fe9_480w.cmfv

http3 request has different header:

Header: range
Value in http3:
bytes=135256-282923

it is a video stream format, pinterest write requesting bytes into HTTP3 header



5.
While using app: com.best.quick.browser
Uri: https://ssp.wknd.ai/magnite

http3 request has distinct header:

Http3 specific Header: content-type

Captured value of this header:
application/json

http3 request has specific body:
{"id":"1730772720830272_4","site":{"mobile":1,"page":"https://www.ksl.com/article/51180604/finding-common-ground-jennie-taylor-touts-unity-amid-political-discord-as-veterans-week-activities-start","name":"Deseret | KSL","domain":"www.ksl.com","privacypolicy":1,"publisher":{"domain":"www.ksl.com","name":"Deseret | KSL","ext":{"rp":{"account_id":20986}}},"ext":{"rp":{"site_id":535010}}},"device":{"ua":"Mozilla/5.0 (Linux; Android 13; Pixel 6 Build/TQ3A.230901.001; wv) AppleWebKit/537.36 (KHTML, like Gecko) Version/4.0 Chrome/129.0.6668.100 Mobile Safari/537.36","ip":"132.205.228.81","js":1,"language":"EN"},"imp":[{"id":"7","banner":{"w":720,"h":480,"ext":{"rp":{"size_id":148,"mime":"text/html"}}},"ext":{"gpid":"masthead","rp":{"zone_id":3290632,"target":{"website_id":[3451]}}}}],"at":1,"tmax":300,"user":{"eids":[{"source":"wunderkind.co","uids":[{"id":"2oPYY5pe4zkd0kSJPOqcTygq372","atype":1}]},{"source":"33across.com","uids":[{"id":"v1.0015a0000344WLQAA2.1035.pTKVjfj2Y3fCSLRfn3hru19t6oB3piwdDHYvHDDa1XWn1wuy8a9C38V9uOeT5OWVd28HMjSSv8GQfIUFHLjXilRo++0hW63RWmlVix+R2OpszW5Udi47EsG9RmiXUb1ODENppwZ8W9iNLGrUSVN/j1pZKYF3jYGpIa6pxSE/BV2kLmv3tbKuF969KqbCj/q+pkMuGLnyQ1ynbdlgBHcO7LcNNUQtk+8zh3eDCdXo9dmpInn6dIbU+FXbM9nfGKz8/2zBH707+O1TzaKZF+EX3houHeyAmVuPTrF2RgPwmYTDkhkv6jrC/qodVDATqi/R69SLx7LzOarLPBavp00RAHEfpLN6a3f2hKaUHIQxNrBHutXclrDJshsqlJ9BLxy4+PSNQPTaaRF023XsEifHoA2aA9vZ+FXPeaLnJnRztIodzvBtnTiI6h3V5fT/LHlhmkW2uqwz9fvBjXZiXxe7lACegVE9L5J/kbWZHCgAz2B8r8VWlT5C1rFpl5PTTJ+/f+bM1gzkOzJ0IaSYVWPdxs+7NrTsvJed/LdvZ8vfOnFWmYqIeyaO/LFhQCFi7l09aDHaWaUVz01lj6vzZAClB0S0d2Av/pOLQ18QPW/UuNoWGZKZvXUVMz0UIP/VvINuJfa3aasciuiXeL/bL3SJXU+nUfuKUP4CmTmI/I0W1eP9Ar/boJ8mCF78DQYW8uoMXmHHTOLp2IFNZlwoPl5WSCKNOFYicJno3Gsx771GwotNcqS2di7pUTUjGiIHS2NFftHCR3+YltUgf7nRLOaoNHn/LJEFDRJO6XPK034tG2e+d1FAMNEU3JdHRtBR6WRf08NO9gCCR3a62s2gF4Ovlg42qX+wk+EceKvgjxXr92C+eKxcHW3WoN5+qc8aH8ypspWTBfum2PrAVxjYYLpw8T+10oqRpBjanFcHSrqGNC5cowW01nR2auMtVQtWFix2FOd1s/ZIb95ybnKdfZNuzwCZn8bFfTqjU1puWq6DXSmaALKrRNSEbZZ84w4h4zwghcGyVNYM6G76YTCB/0WPgzoCZmBWxow5GkuXWbWCu4PQ5X6kk3cEdPpoYzHXk/efZfJpqMllJNNOdpaF3LL4MRo5wRNQv60wfjexr5y+yDpJwfYB1hNRRGQSo388O1sxAQ8EvFMmb9bz+3MymSd+Df0hlK1VefItvBjnda7WuV1d8SCoNwrUgs5a2cZkM6+yLDVP8ZKd5fpRedHhe81f+rNBStuw5WpP01TztiO9DerRTmj0sjAXqaxWibHPQQ0VFJR7nNooxK0by6247Ehm779rRQOVx7tvDGML64ovqjeorrLkkcV4q1EEcHPpMWzZ3CT7hJq6H6sUz5cXao832m0rAeBlteTXGDqDwNvrjgQk+dPsZTAJG/Epn4bbUCTMN9fdMFLqQyOw+DODqhp/MKXDxg13TuVLiDlbzYkxRwTWJonSS+fnGcPYS0Tn0ckn3WaVsl2JCojsR0qWkc1BDTAnOUz8obKys9E/DLQ0DL/ygaMQyAKmKYARbGz1Qngj+OuzTiUwicdGT7nVWhDhjQ==","atype":1}]},{"source":"criteo.com","uids":[{"id":"ksFnyV9PeHhnU2tzaXFqWlY2cnNiS054TVd6YlZCQ1UlMkJ4Umd1b2p0TVhuRTJvQlJDZHBCV1E2SW5KTHU3VmgzQzdqTE1oOUklMkJvYlVvWXBqM245OW9INTFkcmclM0QlM0Q","atype":1}]},{"source":"pubcid.org","uids":[{"id":"affa2070-ae84-4963-99fa-b84fe14f0b1e","atype":1}]},{"source":"adserver.org","uids":[{"id":"79e7d2b7-cbbb-4000-afdd-d7b46210c953","atype":1,"ext":{"rtiPartner":"TDID"}}]}]},"source":{"ext":{"schain":{"ver":"1.0","complete":1,"nodes":[{"asi":"wunderkind.co","sid":"3451","hp":1,"rid":"1730772720830272_4","name":"Deseret | KSL","domain":"ksl.com"}]}}},"regs":{"ext":{"gdpr":0,"us_privacy":"1---"}}}

after formatting:
{
  "id": "1730772720830272_4",
  "site": {
    "mobile": 1,
    "page": "https://www.ksl.com/article/51180604/finding-common-ground-jennie-taylor-touts-unity-amid-political-discord-as-veterans-week-activities-start",
    "name": "Deseret | KSL",
    "domain": "www.ksl.com",
    "privacypolicy": 1,
    "publisher": {
      "domain": "www.ksl.com",
      "name": "Deseret | KSL",
      "ext": {
        "rp": {
          "account_id": 20986
        }
      }
    },
    "ext": {
      "rp": {
        "site_id": 535010
      }
    }
  },
  "device": {
    "ua": "Mozilla/5.0 (Linux; Android 13; Pixel 6 Build/TQ3A.230901.001; wv) AppleWebKit/537.36 (KHTML, like Gecko) Version/4.0 Chrome/129.0.6668.100 Mobile Safari/537.36",
    "ip": "132.205.228.81",
    "js": 1,
    "language": "EN"
  },
  "imp": [{
    "id": "7",
    "banner": {
      "w": 720,
      "h": 480,
      "ext": {
        "rp": {
          "size_id": 148,
          "mime": "text/html"
        }
      }
    },
    "ext": {
      "gpid": "masthead",
      "rp": {
        "zone_id": 3290632,
        "target": {
          "website_id": [3451]
        }
      }
    }
  }],
  "at": 1,
  "tmax": 300,
  "user": {
    "eids": [
      {
        "source": "wunderkind.co",
        "uids": [{
          "id": "2oPYY5pe4zkd0kSJPOqcTygq372",
          "atype": 1
        }]
      },
      {
        "source": "33across.com",
        "uids": [{
          "id": "v1.0015a0000344WLQAA2.1035.[long-identifier]",
          "atype": 1
        }]
      },
      {
        "source": "criteo.com",
        "uids": [{
          "id": "ksFnyV9PeHhnU2tzaXFqWlY2cnNiS054TVd6YlZCQ1UlMkJ4Umd1b2p0TVhuRTJvQlJDZHBCV1E2SW5KTHU3VmgzQzdqTE1oOUklMkJvYlVvWXBqM245OW9INTFkcmclM0QlM0Q",
          "atype": 1
        }]
      },
      {
        "source": "pubcid.org",
        "uids": [{
          "id": "affa2070-ae84-4963-99fa-b84fe14f0b1e",
          "atype": 1
        }]
      },
      {
        "source": "adserver.org",
        "uids": [{
          "id": "79e7d2b7-cbbb-4000-afdd-d7b46210c953",
          "atype": 1,
          "ext": {
            "rtiPartner": "TDID"
          }
        }]
      }
    ]
  },
  "source": {
    "ext": {
      "schain": {
        "ver": "1.0",
        "complete": 1,
        "nodes": [{
          "asi": "wunderkind.co",
          "sid": "3451",
          "hp": 1,
          "rid": "1730772720830272_4",
          "name": "Deseret | KSL",
          "domain": "ksl.com"
        }]
      }
    }
  },
  "regs": {
    "ext": {
      "gdpr": 0,
      "us_privacy": "1---"
    }
  }
}

In this scenario, the app is a browser containing some ads. The app sends http2 OPTIONs requests without body,
but these requests has header fields:
- access-control-request-headers: "content-type"
- access-control-request-method: "POST"
pointing to the http3 POST requests. After we check those http3 POST requests body, we have the above info,
including ads service and the identifier they provide.




