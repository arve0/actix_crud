SELECT
    cast(strftime('%Y', date) as int) as theyear,
    cast(strftime('%m', date) as int) as themonth
    ,sum(tmax) as total
FROM nyc_weather
WHERE tmax <> 'TMAX'
GROUP BY theyear, themonth
ORDER BY total DESC LIMIT 10
;