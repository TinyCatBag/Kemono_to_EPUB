# Kemono_to_EPUB
Scrapes kemono creators/posts and makes an epub out of them.
# How to use
./kemono_to_epub -c {creator url}<br>
Example<br>
./kemono_to_epub -c https://kemono.su/patreon/user/31891971?o=50<br>
Then just follow the tui<br>
thx for reading
# Custom title
You can insert first and last posts title and total count with: <br>
<ul>
  <li>{Posts.first}</li>
  <li>{Posts.last}</li>
  <li>{Posts.count}</li>
</ul>
<hr>
It's also possible to insert the creator's name<br>
  <li><ul>{Creator.name}</li></ul>
<hr>
It's possible to escape this by simply adding \ after the first {. <br>
  <ul><li>{\Posts.first}</li></ul>
<hr>
It's case sensitive for now.<br>
