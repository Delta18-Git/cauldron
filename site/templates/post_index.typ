#let post-index(posts: array, meta: dictionary) = body => {
  set document(
    title: meta.at("title", default: "Posts"),
    author: meta.at("author", default: ""),
    description: meta.at("description", default: none),
  )

  html.link(rel: "stylesheet", href: "../static/style.css")

  html.nav()[
    #html.a(href: "../tags/index.html")[Tags]
  ]

  html.main()[
    #html.h1()[#meta.at("title", default: "Posts")]

    #for post in posts [
      #html.article()[
        #html.h2()[
          #html.a(href: post.link)[
            #post.frontmatter.at("title", default: "Untitled")
          ]
        ]

        #if post.frontmatter.at("author", default: none) != none [
          #html.address()[#post.frontmatter.at("author", default: none)]
        ]

        #if post.frontmatter.at("description", default: none) != none [
          #html.p()[#post.frontmatter.at("description", default: none)]
        ]

        #let tags = post.frontmatter.at("tags", default: ());
        #if tags != () and tags.len() > 0 [
          #html.ul()[
            #for tag in tags [
              #html.li()[
                #html.a(href: "../tags/" + tag + ".html")[#tag]
              ]
            ]
          ]
        ]
      ]
    ]

    #body
  ]
}

#show: post-index(posts: sys.inputs.at("posts", default: (:)), meta: sys.inputs.at("meta", default: (:)))
