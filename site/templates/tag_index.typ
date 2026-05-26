#let tag-overview(tags: array, meta: dictionary) = body => {
  set document(
    title: meta.at("title", default: "Tags"),
    description: meta.at("description", default: none),
  )

  html.link(rel: "stylesheet", href: "../static/style.css")

  html.nav()[
    #html.a(href: "../posts/index.html")[Posts]
  ]

  html.main()[
    #html.h1()[#meta.at("title", default: "Tags")]

    #html.ul()[
      #for entry in tags [
        #html.li()[
          #html.a(href: entry.name + ".html")[
            #entry.name
            #text(" (")
            #entry.posts.len()
            #text(")")
          ]
        ]
      ]
    ]

    #body
  ]
}

#let tag-page(tag: str, posts: array, meta: dictionary) = body => {
  set document(
    title: meta.at("title", default: tag),
    description: meta.at("description", default: none),
  )

  html.link(rel: "stylesheet", href: "../static/style.css")

  html.nav()[
    #html.a(href: "../posts/index.html")[Posts]
    #text(" · ")
    #html.a(href: "index.html")[All Tags]
  ]

  html.main()[
    #html.h1()[
      #text("Tag: ")
      #tag
    ]

    #if meta.at("description", default: none) != none [
      #html.p()[#meta.at("description", default: none)]
    ]

    #for post in posts [
      #html.article()[
        #html.h2()[
          #html.a(href: "../" + post.link)[
            #post.frontmatter.at("title", default: "Untitled")
          ]
        ]

        #if post.frontmatter.at("author", default: none) != none [
          #html.address()[#post.frontmatter.at("author", default: none)]
        ]

        #if post.frontmatter.at("description", default: none) != none [
          #html.p()[#post.frontmatter.at("description", default: none)]
        ]
      ]
    ]

    #body
  ]
}

#show: tag-page(
  tag: sys.inputs.at("tag", default: "unknown"),
  posts: sys.inputs.at("posts", default: (:)),
  meta: sys.inputs.at("meta", default: (:)),
)
