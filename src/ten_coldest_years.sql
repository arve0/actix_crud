SELECT cast(strftime('%Y', date) as int) as theyear,
        sum(tmax) as total
FROM nyc_weather
WHERE tmax <> 'TMAX'
GROUP BY theyear
ORDER BY total ASC LIMIT 10;