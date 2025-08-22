# <center>Xiaoyaou's Github Pages</center>

<div class="post-cards">
{% for post in site.posts %}
  <div class="post-card">
    <h3><a href="{{ post.url }}">{{ post.title }}</a></h3>
    <div class="post-meta-index">
      <span class="post-date">{{ post.date | date: "%Y-%m-%d" }}</span>
      <div class="post-tags">
        {% for tag in post.tags %}
          <span class="tag">{{ tag }}</span>
        {% endfor %}
      </div>
    </div>
  </div>
{% endfor %}
</div>